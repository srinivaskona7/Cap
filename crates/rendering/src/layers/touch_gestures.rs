use cap_project::XY;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TouchRippleSample {
    pub x: f32,
    pub y: f32,
    pub age_sec: f32,
    pub duration_sec: f32,
}

impl TouchRippleSample {
    pub fn radius_px(&self) -> f32 {
        let progress = (self.age_sec / self.duration_sec.max(0.05)).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);
        12.0 + (36.0 - 12.0) * eased
    }

    pub fn opacity(&self) -> f32 {
        let progress = (self.age_sec / self.duration_sec.max(0.05)).clamp(0.0, 1.0);
        (0.85 * (1.0 - progress * progress)).clamp(0.0, 0.85)
    }
}

pub fn render_touch_ripple_svg(
    width: u32,
    height: u32,
    ripples: &[TouchRippleSample],
    trail: &[XY<f32>],
    pinch_pair: Option<(XY<f32>, XY<f32>)>,
) -> String {
    let mut elements = String::new();
    for r in ripples {
        let rad = r.radius_px();
        let alpha = r.opacity();
        if alpha <= 0.01 {
            continue;
        }
        let inner_r = (rad * 0.42).max(4.0);
        elements.push_str(&format!(
            concat!(
                r##"<circle cx="{x}" cy="{y}" r="{rad}" fill="rgba(56,189,248,{fill_a})" stroke="rgba(56,189,248,{alpha})" stroke-width="2.5"/>"##,
                r##"<circle cx="{x}" cy="{y}" r="{inner_r}" fill="rgba(255,255,255,{alpha})"/>"##,
            ),
            x = r.x,
            y = r.y,
            rad = rad,
            fill_a = alpha * 0.22,
            alpha = alpha,
            inner_r = inner_r,
        ));
    }

    if trail.len() >= 2 {
        let mut path_d = String::new();
        for (idx, pt) in trail.iter().enumerate() {
            if idx == 0 {
                path_d.push_str(&format!("M {} {}", pt.x, pt.y));
            } else {
                path_d.push_str(&format!(" L {} {}", pt.x, pt.y));
            }
        }
        elements.push_str(&format!(
            r##"<path d="{path_d}" fill="none" stroke="rgba(56,189,248,0.78)" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/>"##,
        ));
    }

    if let Some((a, b)) = pinch_pair {
        elements.push_str(&format!(
            concat!(
                r##"<line x1="{ax}" y1="{ay}" x2="{bx}" y2="{by}" stroke="rgba(167,139,250,0.85)" stroke-width="2.5" stroke-dasharray="5,4"/>"##,
                r##"<circle cx="{ax}" cy="{ay}" r="16" fill="rgba(167,139,250,0.28)" stroke="#A78BFA" stroke-width="2.5"/>"##,
                r##"<circle cx="{bx}" cy="{by}" r="16" fill="rgba(167,139,250,0.28)" stroke="#A78BFA" stroke-width="2.5"/>"##,
            ),
            ax = a.x,
            ay = a.y,
            bx = b.x,
            by = b.y,
        ));
    }

    format!(
        r##"<svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">{elements}</svg>"##
    )
}

pub fn embed_forensic_luma_watermark(rgba: &mut [u8], payload: &[u8]) {
    let total_bits = payload.len() * 8;
    for (bit_idx, px) in rgba.chunks_exact_mut(4).take(total_bits).enumerate() {
        let byte = payload[bit_idx / 8];
        let bit = (byte >> (7 - (bit_idx % 8))) & 1;
        px[0] = (px[0] & !1) | bit;
    }
}

pub fn extract_forensic_luma_watermark(rgba: &[u8], byte_len: usize) -> Vec<u8> {
    let mut out = vec![0u8; byte_len];
    let total_bits = byte_len * 8;
    for (bit_idx, px) in rgba.chunks_exact(4).take(total_bits).enumerate() {
        let bit = px[0] & 1;
        out[bit_idx / 8] |= bit << (7 - (bit_idx % 8));
    }
    out
}

pub fn generate_c2pa_provenance_manifest_json(
    title: &str,
    sha256_hex: &str,
    author: &str,
) -> String {
    serde_json::json!({
        "claim_generator": "sricap-rec/0.6.8 (Apple Silicon Metal + Secure Enclave C2PA)",
        "title": title,
        "author": author,
        "assertions": [
            {
                "label": "c2pa.actions",
                "action": "c2pa.recorded",
                "digitalSourceType": "http://cv.iptc.org/newscodes/digitalsourcetype/screenCapture"
            },
            {
                "label": "c2pa.hash.data",
                "alg": "sha256",
                "hash": sha256_hex
            }
        ]
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_ripple_expands_from_12px_to_36px_and_fades() {
        let start = TouchRippleSample {
            x: 120.0,
            y: 240.0,
            age_sec: 0.0,
            duration_sec: 0.5,
        };
        assert!((start.radius_px() - 12.0).abs() < 1e-4);
        assert!((start.opacity() - 0.85).abs() < 1e-4);

        let end = TouchRippleSample {
            x: 120.0,
            y: 240.0,
            age_sec: 0.5,
            duration_sec: 0.5,
        };
        assert!((end.radius_px() - 36.0).abs() < 1e-4);
        assert!(end.opacity() < 1e-4);

        let svg = render_touch_ripple_svg(
            400,
            800,
            &[start],
            &[XY::new(10.0, 20.0), XY::new(80.0, 120.0)],
            Some((XY::new(100.0, 200.0), XY::new(220.0, 260.0))),
        );
        assert!(svg.contains("<circle"));
        assert!(svg.contains("<path"));
        assert!(svg.contains("<line"));
    }

    #[test]
    fn forensic_luma_watermark_round_trips_and_c2pa_manifest_validates() {
        let mut frame = vec![128u8; 64 * 4];
        let secret = b"SRICAP01";
        embed_forensic_luma_watermark(&mut frame, secret);
        let recovered = extract_forensic_luma_watermark(&frame, secret.len());
        assert_eq!(recovered, secret);

        let manifest = generate_c2pa_provenance_manifest_json(
            "Demo",
            "e3b0c44298fc1c149afbf4c8996fb924",
            "kona_srinivas",
        );
        assert!(manifest.contains("c2pa.recorded"));
        assert!(manifest.contains("screenCapture"));
    }
}
