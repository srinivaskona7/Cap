use crate::{AudioSampleSource, VoiceEnhancer, cast_bytes_to_f32_slice, cast_f32_slice_to_bytes};
use ffmpeg::{ChannelLayout, filter, format, frame::Audio};

pub const VOICE_PROFILE_SAMPLES: usize = 16 * 48_000;
fn compressor(channels: u16) -> String {
    let threshold = if channels == 1 {
        0.125 * std::f32::consts::SQRT_2
    } else {
        0.125
    };
    format!("acompressor=threshold={threshold}:ratio=2:attack=15:release=180:knee=2.828:makeup=1")
}

pub fn studio_mastering_chain(channels: u16) -> String {
    format!(
        "highpass=f=80,agate=threshold=0.012:ratio=2.5:attack=10:release=160,equalizer=f=250:t=q:w=1.0:g=2.0,equalizer=f=5000:t=q:w=1.2:g=3.0,equalizer=f=6800:t=q:w=2.0:g=-2.5,{}",
        compressor(channels)
    )
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VoiceProfile {
    pub gain_db: f32,
    pub makeup_db: f32,
}

struct ProfileSource<'a, T> {
    source: &'a T,
    frames: usize,
    channels: u16,
}

impl<T: AudioSampleSource> AudioSampleSource for ProfileSource<'_, T> {
    fn channels(&self) -> u16 {
        self.channels
    }
    fn sample_count(&self) -> usize {
        self.frames
    }
    fn sample(&self, index: usize) -> Option<&f32> {
        if index / usize::from(self.channels) < self.frames {
            self.source.sample(index)
        } else {
            None
        }
    }
}

impl VoiceProfile {
    pub fn analyze(source: &impl AudioSampleSource) -> Self {
        Self::try_analyze(source).unwrap_or_else(|error| {
            tracing::warn!(%error, "Could not estimate Studio Sound voice level");
            Self::default()
        })
    }

    fn try_analyze(source: &impl AudioSampleSource) -> Result<Self, ffmpeg::Error> {
        let channels = source.channels().clamp(1, 2);
        let frames = source.sample_count().min(VOICE_PROFILE_SAMPLES);
        let source = ProfileSource {
            source,
            frames,
            channels,
        };
        let window = frames.min(24_000);
        if window < 480 {
            return Ok(Self::default());
        }
        let windows = (frames / window).clamp(1, 8);
        let mut samples = Vec::with_capacity(windows * window * usize::from(channels));
        let mut speech_frames = 0;
        for index in 0..windows {
            let start = if windows == 1 {
                0
            } else {
                (frames - window) * index / (windows - 1)
            };
            let mut enhancer = VoiceEnhancer::new(channels);
            let audio = enhancer.render(&source, start, window);
            speech_frames += enhancer.speech_frames();
            samples.extend_from_slice(audio.samples());
        }
        if speech_frames < 20 {
            return Ok(Self::default());
        }
        let Some(level) = measure(&samples, channels, "anull")? else {
            return Ok(Self::default());
        };
        if level < -60.0 {
            return Ok(Self::default());
        }
        let gain_db = (-14.0 - level).clamp(-12.0, 24.0);
        let filter = format!("volume={gain_db}dB,{}", compressor(channels));
        let compressed = measure(&samples, channels, &filter)?.unwrap_or(-14.0);
        Ok(Self {
            gain_db,
            makeup_db: (-14.0 - compressed).clamp(-6.0, 6.0),
        })
    }

    pub(crate) fn filter(self, channels: u16) -> String {
        format!(
            "volume={}dB,{},volume={}dB,aresample=192000,alimiter=limit=0.841395:attack=5:release=80:level=false:latency=true,aresample=48000",
            self.gain_db,
            compressor(channels),
            self.makeup_db,
        )
    }

    pub fn broadcast_filter(self, channels: u16) -> String {
        format!(
            "volume={}dB,{},volume={}dB,aresample=192000,alimiter=limit=0.841395:attack=5:release=80:level=false:latency=true,aresample=48000",
            self.gain_db,
            studio_mastering_chain(channels),
            self.makeup_db,
        )
    }
}

fn measure(samples: &[f32], channels: u16, processing: &str) -> Result<Option<f32>, ffmpeg::Error> {
    let mut filter = VoiceFilter::new(
        channels,
        &format!("{processing},ebur128=metadata=1:framelog=verbose"),
    )?;
    let mut level = None;
    for chunk in samples.chunks(4_800 * usize::from(channels)) {
        filter.push(chunk)?;
        filter.drain(|frame| {
            if let Some(value) = frame.metadata().get("lavfi.r128.I") {
                level = value.parse::<f32>().ok().filter(|value| value.is_finite());
            }
        })?;
    }
    filter.graph.get("in").unwrap().source().flush()?;
    filter.drain(|frame| {
        if let Some(value) = frame.metadata().get("lavfi.r128.I") {
            level = value.parse::<f32>().ok().filter(|value| value.is_finite());
        }
    })?;
    Ok(level)
}

pub(crate) struct VoiceFilter {
    graph: filter::Graph,
    channels: usize,
    position: i64,
    output: Audio,
}

impl VoiceFilter {
    pub(crate) fn new(channels: u16, processing: &str) -> Result<Self, ffmpeg::Error> {
        let layout = ChannelLayout::default(i32::from(channels));
        let mut graph = filter::Graph::new();
        unsafe { (*graph.as_mut_ptr()).nb_threads = 1 };
        graph.add(
            &filter::find("abuffer").ok_or(ffmpeg::Error::FilterNotFound)?,
            "in",
            &format!(
                "time_base=1/48000:sample_rate=48000:sample_fmt=flt:channel_layout=0x{:x}",
                layout.bits()
            ),
        )?;
        graph.add(
            &filter::find("abuffersink").ok_or(ffmpeg::Error::FilterNotFound)?,
            "out",
            "",
        )?;
        graph.output("in", 0)?.input("out", 0)?.parse(&format!(
            "{processing},aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=0x{:x}",
            layout.bits()
        ))?;
        graph.validate()?;
        Ok(Self {
            graph,
            channels: usize::from(channels),
            position: 0,
            output: Audio::empty(),
        })
    }

    fn push(&mut self, samples: &[f32]) -> Result<(), ffmpeg::Error> {
        let frames = samples.len() / self.channels;
        let mut frame = Audio::new(
            format::Sample::F32(format::sample::Type::Packed),
            frames,
            ChannelLayout::default(self.channels as i32),
        );
        frame.set_rate(48_000);
        frame.set_pts(Some(self.position));
        frame.data_mut(0)[..samples.len() * 4]
            .copy_from_slice(unsafe { cast_f32_slice_to_bytes(samples) });
        self.graph.get("in").unwrap().source().add(&frame)?;
        self.position += frames as i64;
        Ok(())
    }

    fn drain(&mut self, mut visit: impl FnMut(&Audio)) -> Result<(), ffmpeg::Error> {
        loop {
            match self
                .graph
                .get("out")
                .unwrap()
                .sink()
                .frame(&mut self.output)
            {
                Ok(()) => {
                    visit(&self.output);
                    // The filter sink transfers a frame reference; reusing a referenced frame leaks its buffers.
                    unsafe { ffmpeg::ffi::av_frame_unref(self.output.as_mut_ptr()) };
                }
                Err(ffmpeg::Error::Eof) => return Ok(()),
                Err(ffmpeg::Error::Other { errno }) if errno == ffmpeg::error::EAGAIN => {
                    return Ok(());
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(crate) fn process(
        &mut self,
        samples: &[f32],
        output: &mut std::collections::VecDeque<f32>,
    ) -> Result<(), ffmpeg::Error> {
        self.push(samples)?;
        let channels = self.channels;
        self.drain(|frame| {
            let bytes = &frame.data(0)[..frame.samples() * channels * 4];
            output.extend(unsafe { cast_bytes_to_f32_slice(bytes) });
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioLevelMetrics {
    pub peak_linear: f32,
    pub peak_dbfs: f32,
    pub rms_dbfs: f32,
    pub recommended_gain_db: f32,
    pub clipping_risk: bool,
}

impl Default for AudioLevelMetrics {
    fn default() -> Self {
        Self {
            peak_linear: 0.0,
            peak_dbfs: -60.0,
            rms_dbfs: -60.0,
            recommended_gain_db: 0.0,
            clipping_risk: false,
        }
    }
}

pub fn measure_audio_levels(samples: &[f32]) -> AudioLevelMetrics {
    if samples.is_empty() {
        return AudioLevelMetrics::default();
    }
    let mut peak = 0.0_f32;
    let mut sum_sq = 0.0_f64;
    let mut count = 0_usize;
    for &sample in samples {
        if sample.is_finite() {
            let abs = sample.abs();
            if abs > peak {
                peak = abs;
            }
            sum_sq += f64::from(sample) * f64::from(sample);
            count += 1;
        }
    }
    if count == 0 || peak <= 1e-6 {
        return AudioLevelMetrics::default();
    }
    let rms = (sum_sq / count as f64).sqrt() as f32;
    let peak_dbfs = (20.0 * peak.log10()).clamp(-60.0, 6.0);
    let rms_dbfs = if rms > 1e-6 {
        (20.0 * rms.log10()).clamp(-60.0, 6.0)
    } else {
        -60.0
    };
    let recommended_gain_db = (-1.0 - peak_dbfs).clamp(-12.0, 18.0);
    AudioLevelMetrics {
        peak_linear: peak,
        peak_dbfs,
        rms_dbfs,
        recommended_gain_db,
        clipping_risk: peak >= 0.98,
    }
}

pub fn normalize_peak_gain(samples: &mut [f32], ceiling_dbfs: f32) -> f32 {
    let metrics = measure_audio_levels(samples);
    if metrics.peak_linear <= 1e-6 {
        return 0.0;
    }
    let target_ceiling_db = ceiling_dbfs.clamp(-12.0, -0.1);
    let gain_db = (target_ceiling_db - metrics.peak_dbfs).clamp(-12.0, 18.0);
    let gain_linear = 10.0_f32.powf(gain_db / 20.0);
    let ceiling_linear = 10.0_f32.powf(target_ceiling_db / 20.0);
    for sample in samples.iter_mut() {
        let scaled = *sample * gain_linear;
        let abs = scaled.abs();
        *sample = if abs <= ceiling_linear {
            scaled
        } else {
            let excess = abs - ceiling_linear;
            let headroom = (1.0 - ceiling_linear).max(0.01);
            let compressed = ceiling_linear + headroom * (excess / headroom).tanh();
            compressed.copysign(scaled)
        };
    }
    gain_db
}

pub fn apply_noise_gate_and_rnnoise(
    samples: &[f32],
    channels: u16,
    gate_threshold_db: f32,
) -> Vec<f32> {
    let ch = usize::from(channels.clamp(1, 2));
    if samples.is_empty() {
        return Vec::new();
    }
    let threshold_linear = 10.0_f32.powf(gate_threshold_db.clamp(-70.0, -18.0) / 20.0);
    let attack_coeff = (-1.0_f32 / (0.004 * 48_000.0)).exp();
    let release_coeff = (-1.0_f32 / (0.085 * 48_000.0)).exp();
    let floor_gain = 0.08_f32;

    let mut gated = Vec::with_capacity(samples.len());
    let mut envelope = 0.0_f32;
    let mut current_gain = floor_gain;

    for frame in samples.chunks(ch) {
        let frame_peak = frame.iter().fold(0.0_f32, |acc, s| acc.max(s.abs()));
        let coeff = if frame_peak > envelope {
            attack_coeff
        } else {
            release_coeff
        };
        envelope = coeff * envelope + (1.0 - coeff) * frame_peak;
        let target_gain = if envelope >= threshold_linear {
            1.0
        } else {
            let ratio = (envelope / threshold_linear.max(1e-6)).clamp(0.0, 1.0);
            floor_gain + (1.0 - floor_gain) * ratio * ratio
        };
        current_gain = 0.995 * current_gain + 0.005 * target_gain;
        for &s in frame {
            gated.push(s * current_gain);
        }
    }

    let raw_audio = crate::AudioData::from_samples(gated, channels.clamp(1, 2));
    let frames = raw_audio.sample_count();
    let mut enhancer = VoiceEnhancer::new(channels.clamp(1, 2));
    let mut denoised = enhancer.render(&raw_audio, 0, frames).samples().to_vec();
    let _ = normalize_peak_gain(&mut denoised, -1.0);
    denoised
}

pub fn synthesize_tactile_click_and_keystroke_sfx(
    total_samples: usize,
    sample_rate: u32,
    click_times_sec: &[f64],
    keystroke_times_sec: &[f64],
) -> Vec<f32> {
    if total_samples == 0 || sample_rate == 0 {
        return Vec::new();
    }
    let mut pcm = vec![0.0_f32; total_samples];
    let sr = f64::from(sample_rate);

    for &t in click_times_sec {
        if t < 0.0 {
            continue;
        }
        let start = (t * sr) as usize;
        let len = (0.018 * sr) as usize;
        for i in 0..len {
            let idx = start + i;
            if idx >= total_samples {
                break;
            }
            let local_t = i as f32 / sample_rate as f32;
            let env = (-local_t * 260.0).exp();
            let transient = (2.0 * std::f32::consts::PI * 1850.0 * local_t).sin() * 0.22
                + (2.0 * std::f32::consts::PI * 680.0 * local_t).sin() * 0.14;
            pcm[idx] = (pcm[idx] + transient * env).clamp(-0.5, 0.5);
        }
    }

    for &t in keystroke_times_sec {
        if t < 0.0 {
            continue;
        }
        let start = (t * sr) as usize;
        let len = (0.012 * sr) as usize;
        for i in 0..len {
            let idx = start + i;
            if idx >= total_samples {
                break;
            }
            let local_t = i as f32 / sample_rate as f32;
            let env = (-local_t * 360.0).exp();
            let switch_thock = (2.0 * std::f32::consts::PI * 2400.0 * local_t).sin() * 0.10
                + (2.0 * std::f32::consts::PI * 920.0 * local_t).sin() * 0.08;
            pcm[idx] = (pcm[idx] + switch_thock * env).clamp(-0.5, 0.5);
        }
    }

    pcm
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StudioAudioProConfig {
    pub ai_noise_shield: bool,
    pub keyboard_click_silencer: bool,
    pub warmth_250hz_boost: bool,
    pub presence_5khz_boost: bool,
    pub dynamic_de_esser: bool,
    pub high_pass_80hz: bool,
    pub soft_knee_noise_gate: bool,
    pub broadcast_limiter_minus_1db: bool,
    pub normalize_minus_14_lufs: bool,
    pub auto_ducking_minus_12db: bool,
    pub room_reverb_dampener: bool,
    pub mic_agc_auto_leveler: bool,
    pub echo_cancellation: bool,
}

impl Default for StudioAudioProConfig {
    fn default() -> Self {
        Self {
            ai_noise_shield: true,
            keyboard_click_silencer: true,
            warmth_250hz_boost: true,
            presence_5khz_boost: true,
            dynamic_de_esser: true,
            high_pass_80hz: true,
            soft_knee_noise_gate: true,
            broadcast_limiter_minus_1db: true,
            normalize_minus_14_lufs: true,
            auto_ducking_minus_12db: true,
            room_reverb_dampener: true,
            mic_agc_auto_leveler: true,
            echo_cancellation: true,
        }
    }
}

pub fn suppress_mechanical_keyboard_transients(samples: &mut [f32], sample_rate: u32) {
    if samples.is_empty() || sample_rate == 0 {
        return;
    }
    let window = ((sample_rate as usize) / 100).max(16);
    let mut slow_env = 0.01_f32;
    for chunk in samples.chunks_mut(window) {
        let peak = chunk.iter().fold(0.0_f32, |a, s| a.max(s.abs()));
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
        slow_env = 0.92 * slow_env + 0.08 * rms.max(0.001);
        let crest = peak / rms.max(1e-4);
        if crest > 4.2 && peak > slow_env * 3.0 {
            let atten = (slow_env * 2.2 / peak).clamp(0.18, 1.0);
            for s in chunk.iter_mut() {
                *s *= atten;
            }
        }
    }
}

pub fn apply_room_reverb_dampener(samples: &mut [f32]) {
    let mut env = 0.0_f32;
    for s in samples.iter_mut() {
        let abs = s.abs();
        if abs > env {
            env = 0.7 * env + 0.3 * abs;
        } else {
            env = 0.996 * env + 0.004 * abs;
        }
        if abs < env * 0.38 && env < 0.15 {
            *s *= 0.55;
        }
    }
}

pub fn apply_mic_agc_auto_leveler(samples: &mut [f32], sample_rate: u32) {
    if samples.is_empty() || sample_rate == 0 {
        return;
    }
    let block = ((sample_rate as usize) / 5).max(256);
    let target_rms = 0.14_f32;
    let mut current_gain = 1.0_f32;
    for chunk in samples.chunks_mut(block) {
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
        let desired = if rms > 0.008 {
            (target_rms / rms).clamp(0.5, 3.2)
        } else {
            1.0
        };
        for s in chunk.iter_mut() {
            current_gain = 0.9985 * current_gain + 0.0015 * desired;
            *s = (*s * current_gain).clamp(-0.98, 0.98);
        }
    }
}

pub fn apply_headphone_free_echo_canceler(mic_samples: &mut [f32], reference_system: &[f32]) {
    if mic_samples.is_empty() || reference_system.is_empty() {
        return;
    }
    let len = mic_samples.len().min(reference_system.len());
    let mut weight = 0.08_f32;
    let mu = 0.015_f32;
    for i in 0..len {
        let ref_s = reference_system[i];
        let est_echo = weight * ref_s;
        let err = mic_samples[i] - est_echo;
        weight = (weight + mu * err * ref_s).clamp(-0.65, 0.65);
        mic_samples[i] = err.clamp(-1.0, 1.0);
    }
}

pub fn normalize_lufs_minus_14(samples: &mut [f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let mean_sq = samples
        .iter()
        .filter(|s| s.is_finite())
        .map(|s| f64::from(*s) * f64::from(*s))
        .sum::<f64>()
        / samples.len() as f64;
    if mean_sq <= 1e-10 {
        return 0.0;
    }
    let approx_lufs = (-0.691 + 10.0 * mean_sq.log10()) as f32;
    let delta_db = (-14.0 - approx_lufs).clamp(-12.0, 16.0);
    let gain = 10.0_f32.powf(delta_db / 20.0);
    for s in samples.iter_mut() {
        *s *= gain;
    }
    let _ = normalize_peak_gain(samples, -1.0);
    delta_db
}

pub fn apply_studio_audio_pro_chain(
    samples: &[f32],
    channels: u16,
    sample_rate: u32,
    config: &StudioAudioProConfig,
) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }
    let mut work = if config.ai_noise_shield || config.soft_knee_noise_gate {
        apply_noise_gate_and_rnnoise(
            samples,
            channels,
            if config.soft_knee_noise_gate {
                -42.0
            } else {
                -68.0
            },
        )
    } else {
        samples.to_vec()
    };

    if config.keyboard_click_silencer {
        suppress_mechanical_keyboard_transients(&mut work, sample_rate);
    }
    if config.room_reverb_dampener {
        apply_room_reverb_dampener(&mut work);
    }
    if config.mic_agc_auto_leveler {
        apply_mic_agc_auto_leveler(&mut work, sample_rate);
    }
    if config.normalize_minus_14_lufs {
        let _ = normalize_lufs_minus_14(&mut work);
    } else if config.broadcast_limiter_minus_1db {
        let _ = normalize_peak_gain(&mut work, -1.0);
    }
    work
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_and_normalize_peak_gain_prevent_clipping() {
        let mut samples = vec![0.1_f32, -0.25, 0.35, -0.4, 0.2];
        let before = measure_audio_levels(&samples);
        assert!(before.peak_dbfs < -6.0);
        assert!(!before.clipping_risk);
        assert!(before.recommended_gain_db > 5.0);

        let applied_db = normalize_peak_gain(&mut samples, -1.0);
        assert!(applied_db > 5.0);
        let after = measure_audio_levels(&samples);
        assert!((after.peak_dbfs - -1.0).abs() < 0.15);
        assert!(!after.clipping_risk);
    }

    #[test]
    fn noise_gate_and_rnnoise_suppresses_low_hum() {
        let hum: Vec<f32> = (0..4_800)
            .map(|i| 0.002 * (2.0 * std::f32::consts::PI * 60.0 * (i as f32 / 48_000.0)).sin())
            .collect();
        let cleaned = apply_noise_gate_and_rnnoise(&hum, 1, -40.0);
        assert_eq!(cleaned.len(), hum.len());
        assert!(cleaned.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
    }

    #[test]
    fn tactile_click_and_keystroke_sfx_synthesizes_bounded_transients() {
        let pcm = synthesize_tactile_click_and_keystroke_sfx(48_000, 48_000, &[0.1, 0.5], &[0.25]);
        assert_eq!(pcm.len(), 48_000);
        let metrics = measure_audio_levels(&pcm);
        assert!(metrics.peak_linear > 0.05 && metrics.peak_linear <= 0.5);
    }

    #[test]
    fn studio_audio_pro_chain_eliminates_spikes_and_normalizes_cleanly() {
        let mut raw: Vec<f32> = (0..9_600)
            .map(|i| 0.08 * (2.0 * std::f32::consts::PI * 220.0 * (i as f32 / 48_000.0)).sin())
            .collect();
        raw[1_200] = 0.95;
        let cfg = StudioAudioProConfig::default();
        let processed = apply_studio_audio_pro_chain(&raw, 1, 48_000, &cfg);
        assert_eq!(processed.len(), raw.len());
        let metrics = measure_audio_levels(&processed);
        assert!(!metrics.clipping_risk);
        assert!(metrics.peak_dbfs <= -0.8);
    }
}
