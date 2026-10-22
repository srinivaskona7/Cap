use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl TelemetryRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            width: width.clamp(0.01, 1.0),
            height: height.clamp(0.01, 1.0),
        }
    }

    pub fn center(&self) -> (f64, f64) {
        (
            (self.x + self.width * 0.5).clamp(0.0, 1.0),
            (self.y + self.height * 0.5).clamp(0.0, 1.0),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SemanticElementKind {
    TerminalError,
    ModalDialog,
    IdeBreakpoint,
    NotificationBanner,
    FocusedInput,
    MobileElement,
}

impl SemanticElementKind {
    pub fn suggested_zoom(self) -> f64 {
        match self {
            Self::TerminalError => 1.85,
            Self::ModalDialog => 1.65,
            Self::IdeBreakpoint => 1.75,
            Self::NotificationBanner => 1.55,
            Self::FocusedInput => 1.5,
            Self::MobileElement => 1.6,
        }
    }

    pub fn priority(self) -> u8 {
        match self {
            Self::TerminalError => 100,
            Self::ModalDialog => 90,
            Self::IdeBreakpoint => 85,
            Self::NotificationBanner => 75,
            Self::MobileElement => 70,
            Self::FocusedInput => 60,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticTelemetryPacket {
    pub ts_ms: u64,
    pub element_type: SemanticElementKind,
    pub bounds: TelemetryRect,
    pub text_summary: String,
}

#[derive(Clone, Default)]
pub struct SemanticTelemetryRecorder {
    packets: Arc<Mutex<Vec<SemanticTelemetryPacket>>>,
}

impl SemanticTelemetryRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_packet(&self, packet: SemanticTelemetryPacket) {
        if let Ok(mut guard) = self.packets.lock() {
            guard.push(packet);
        }
    }

    pub fn snapshot(&self) -> Vec<SemanticTelemetryPacket> {
        self.packets.lock().map(|g| g.clone()).unwrap_or_default()
    }

    pub fn save_to_project(&self, project_dir: &Path) -> std::io::Result<()> {
        let packets = self.snapshot();
        let json = serde_json::to_string_pretty(&packets).map_err(std::io::Error::other)?;
        std::fs::write(project_dir.join("semantic-telemetry.json"), json)
    }

    pub fn load_from_project(project_dir: &Path) -> Vec<SemanticTelemetryPacket> {
        let path = project_dir.join("semantic-telemetry.json");
        let Ok(raw) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        serde_json::from_str(&raw).unwrap_or_default()
    }
}

pub fn classify_accessibility_event(role: &str, title_or_value: &str) -> SemanticElementKind {
    let lower_role = role.to_ascii_lowercase();
    let lower_text = title_or_value.to_ascii_lowercase();
    if lower_text.contains("error")
        || lower_text.contains("panic")
        || lower_text.contains("exception")
        || lower_text.contains("failed")
    {
        SemanticElementKind::TerminalError
    } else if lower_role.contains("dialog")
        || lower_role.contains("sheet")
        || lower_role.contains("alert")
        || lower_text.contains("confirm")
    {
        SemanticElementKind::ModalDialog
    } else if lower_text.contains("breakpoint") || lower_role.contains("debugger") {
        SemanticElementKind::IdeBreakpoint
    } else if lower_role.contains("notification") || lower_role.contains("banner") {
        SemanticElementKind::NotificationBanner
    } else {
        SemanticElementKind::FocusedInput
    }
}

pub fn parse_android_accessibility_bounds(
    bounds_str: &str,
    screen_width: f64,
    screen_height: f64,
) -> Option<TelemetryRect> {
    let sw = screen_width.max(1.0);
    let sh = screen_height.max(1.0);
    let cleaned = bounds_str.trim();
    let (first, second) = cleaned.split_once("][")?;
    let first = first.trim_start_matches('[');
    let second = second.trim_end_matches(']');
    let (x1s, y1s) = first.split_once(',')?;
    let (x2s, y2s) = second.split_once(',')?;
    let x1: f64 = x1s.parse().ok()?;
    let y1: f64 = y1s.parse().ok()?;
    let x2: f64 = x2s.parse().ok()?;
    let y2: f64 = y2s.parse().ok()?;
    if x2 <= x1 || y2 <= y1 {
        return None;
    }
    Some(TelemetryRect::new(
        x1 / sw,
        y1 / sh,
        (x2 - x1) / sw,
        (y2 - y1) / sh,
    ))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DvrReplaySample {
    pub ts_ms: u64,
    pub git_branch: String,
    pub git_commit: String,
    pub console_line: String,
    pub active_url_or_window: String,
}

#[derive(Clone)]
pub struct RollingDvrBuffer {
    max_window_ms: u64,
    samples: Arc<Mutex<std::collections::VecDeque<DvrReplaySample>>>,
}

impl Default for RollingDvrBuffer {
    fn default() -> Self {
        Self::new(60_000)
    }
}

impl RollingDvrBuffer {
    pub fn new(max_window_ms: u64) -> Self {
        Self {
            max_window_ms: max_window_ms.max(5_000),
            samples: Arc::new(Mutex::new(std::collections::VecDeque::new())),
        }
    }

    pub fn push(&self, sample: DvrReplaySample) {
        if let Ok(mut guard) = self.samples.lock() {
            let cutoff = sample.ts_ms.saturating_sub(self.max_window_ms);
            while guard.front().is_some_and(|front| front.ts_ms < cutoff) {
                guard.pop_front();
            }
            guard.push_back(sample);
        }
    }

    pub fn snapshot_last_ms(&self, window_ms: u64) -> Vec<DvrReplaySample> {
        let Ok(guard) = self.samples.lock() else {
            return Vec::new();
        };
        let Some(latest) = guard.back().map(|s| s.ts_ms) else {
            return Vec::new();
        };
        let cutoff = latest.saturating_sub(window_ms);
        guard
            .iter()
            .filter(|s| s.ts_ms >= cutoff)
            .cloned()
            .collect()
    }
}

pub fn catmull_rom_point(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let t = t.clamp(0.0, 1.0);
    let t2 = t * t;
    let t3 = t2 * t;
    let interp = |v0: f64, v1: f64, v2: f64, v3: f64| -> f64 {
        0.5 * ((2.0 * v1)
            + (-v0 + v2) * t
            + (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) * t2
            + (-v0 + 3.0 * v1 - 3.0 * v2 + v3) * t3)
    };
    (
        interp(p0.0, p1.0, p2.0, p3.0).clamp(0.0, 1.0),
        interp(p0.1, p1.1, p2.1, p3.1).clamp(0.0, 1.0),
    )
}

pub fn smooth_and_snap_cursor_trajectory(
    raw_points: &[(u64, f64, f64)],
    magnetic_targets: &[TelemetryRect],
    snap_radius: f64,
) -> Vec<(u64, f64, f64)> {
    if raw_points.is_empty() {
        return Vec::new();
    }
    let snapped: Vec<(u64, f64, f64)> = raw_points
        .iter()
        .map(|&(ts, x, y)| {
            let mut best = (x, y);
            let mut best_dist = snap_radius.max(0.005);
            for target in magnetic_targets {
                let (cx, cy) = target.center();
                let dist = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                if dist <= best_dist {
                    best_dist = dist;
                    best = (cx, cy);
                }
            }
            (ts, best.0.clamp(0.0, 1.0), best.1.clamp(0.0, 1.0))
        })
        .collect();

    if snapped.len() < 4 {
        return snapped;
    }

    let mut smoothed = Vec::with_capacity(snapped.len());
    for i in 0..snapped.len() {
        let p0 = snapped[i.saturating_sub(1)];
        let p1 = snapped[i];
        let p2 = snapped[(i + 1).min(snapped.len() - 1)];
        let p3 = snapped[(i + 2).min(snapped.len() - 1)];
        let (sx, sy) =
            catmull_rom_point((p0.1, p0.2), (p1.1, p1.2), (p2.1, p2.2), (p3.1, p3.2), 0.35);
        let blend_x = (p1.1 * 0.65 + sx * 0.35).clamp(0.0, 1.0);
        let blend_y = (p1.2 * 0.65 + sy * 0.35).clamp(0.0, 1.0);
        smoothed.push((p1.0, blend_x, blend_y));
    }
    smoothed
}

pub fn mask_sensitive_keystrokes(display_text: &str, password_context: bool) -> String {
    let trimmed = display_text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.contains('⌘')
        || trimmed.contains('⌃')
        || trimmed.contains('⌥')
        || trimmed.contains("Cmd")
        || trimmed.contains("Ctrl")
    {
        return trimmed.to_string();
    }
    let lower = trimmed.to_ascii_lowercase();
    let is_secret = password_context
        || lower.contains("password")
        || lower.contains("sudo ")
        || lower.contains("sk-")
        || lower.contains("ghp_")
        || lower.contains("secret")
        || lower.contains("bearer ");
    if is_secret {
        "•••••••• (Protected Secret)".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn generate_playwright_e2e_spec(
    title: &str,
    packets: &[SemanticTelemetryPacket],
    commands_or_keys: &[(f64, String)],
) -> String {
    let mut spec = format!(
        "import {{ test, expect }} from \"@playwright/test\";\n\n\
         test(\"{} — automated walkthrough\", async ({{ page }}) => {{\n\
         \tawait page.goto(process.env.APP_URL ?? \"http://localhost:3000\");\n",
        title.replace('"', "'")
    );
    for pkt in packets {
        let (cx, cy) = pkt.bounds.center();
        let summary = pkt.text_summary.replace('"', "'");
        spec.push_str(&format!(
            "\t// [{:.2}s] {:?}: {summary}\n\
             \tawait page.mouse.click(Math.round({cx:.3} * 1280), Math.round({cy:.3} * 720));\n",
            pkt.ts_ms as f64 / 1000.0,
            pkt.element_type
        ));
    }
    for (sec, key_txt) in commands_or_keys {
        let safe = mask_sensitive_keystrokes(key_txt, false).replace('"', "'");
        spec.push_str(&format!(
            "\t// [{sec:.2}s] Keystroke / Input\n\
             \tawait page.keyboard.type(\"{safe}\");\n"
        ));
    }
    spec.push_str("\tawait expect(page).toHaveTitle(/.*/);\n});\n");
    spec
}

pub fn generate_maestro_mobile_flow_yaml(
    app_id: &str,
    packets: &[SemanticTelemetryPacket],
    commands_or_keys: &[(f64, String)],
) -> String {
    let mut yaml = format!("appId: {app_id}\n---\n- launchApp\n");
    for pkt in packets {
        let (cx, cy) = pkt.bounds.center();
        yaml.push_str(&format!(
            "- tapOn:\n    point: \"{:.0}%,{:.0}%\"\n",
            cx * 100.0,
            cy * 100.0
        ));
    }
    for (_, key_txt) in commands_or_keys {
        let safe = mask_sensitive_keystrokes(key_txt, false).replace('"', "'");
        yaml.push_str(&format!("- inputText: \"{safe}\"\n"));
    }
    yaml
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_terminal_errors_and_dialogs_accurately() {
        assert_eq!(
            classify_accessibility_event("AXStaticText", "error[E0599]: method not found"),
            SemanticElementKind::TerminalError
        );
        assert_eq!(
            classify_accessibility_event("AXDialog", "Confirm Merge Pull Request"),
            SemanticElementKind::ModalDialog
        );
    }

    #[test]
    fn parses_android_accessibility_node_bounds_into_normalized_rect() {
        let rect =
            parse_android_accessibility_bounds("[108,480][972,960]", 1080.0, 2400.0).unwrap();
        assert!((rect.x - 0.1).abs() < 1e-6);
        assert!((rect.y - 0.2).abs() < 1e-6);
        let (cx, cy) = rect.center();
        assert!((cx - 0.5).abs() < 1e-6);
        assert!((cy - 0.3).abs() < 1e-6);
    }

    #[test]
    fn dvr_buffer_cursor_snap_keystroke_mask_and_e2e_codegen_work() {
        let dvr = RollingDvrBuffer::new(10_000);
        dvr.push(DvrReplaySample {
            ts_ms: 1_000,
            git_branch: "feature1".into(),
            git_commit: "abc1234".into(),
            console_line: "init".into(),
            active_url_or_window: "localhost:3000".into(),
        });
        dvr.push(DvrReplaySample {
            ts_ms: 15_000,
            git_branch: "feature1".into(),
            git_commit: "def5678".into(),
            console_line: "ready".into(),
            active_url_or_window: "localhost:3000/dashboard".into(),
        });
        let recent = dvr.snapshot_last_ms(10_000);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].git_commit, "def5678");

        let target = TelemetryRect::new(0.45, 0.45, 0.10, 0.10);
        let raw = vec![
            (0, 0.1, 0.1),
            (100, 0.48, 0.49),
            (200, 0.51, 0.50),
            (300, 0.8, 0.8),
        ];
        let smoothed = smooth_and_snap_cursor_trajectory(&raw, &[target], 0.06);
        assert_eq!(smoothed.len(), 4);
        assert!((smoothed[1].1 - 0.5).abs() < 0.05);

        assert_eq!(
            mask_sensitive_keystrokes("sk-proj-1234567890", false),
            "•••••••• (Protected Secret)"
        );
        assert_eq!(mask_sensitive_keystrokes("⌘ + K", true), "⌘ + K");

        let pkt = SemanticTelemetryPacket {
            ts_ms: 1200,
            element_type: SemanticElementKind::ModalDialog,
            bounds: target,
            text_summary: "Submit".into(),
        };
        let pw = generate_playwright_e2e_spec("Checkout", &[pkt.clone()], &[(2.0, "hello".into())]);
        assert!(pw.contains("@playwright/test"));
        assert!(pw.contains("page.mouse.click"));

        let maestro = generate_maestro_mobile_flow_yaml("com.sri.sricap", &[pkt], &[]);
        assert!(maestro.contains("50%,50%"));
    }
}
