# `sricap-rec` Multi-Product Architecture & Open-Source Registry

This document catalogues all verified open-source repositories, architectural bindings, and integration blueprints to transform `sricap-rec` into an all-in-one powerhouse (Screen Recording + Studio Editing + Android Remote Control + AI Audio & Persona Enhancements).

---

## 1. Android Mirroring & Remote Control Subsystem

### Verified Open-Source Repositories
| Component | Repository & Technology | License | Purpose & Role |
| :--- | :--- | :--- | :--- |
| **Core Mirroring & Control** | [Genymobile/scrcpy](https://github.com/Genymobile/scrcpy) (C / Java) | Apache 2.0 | Ultra-low latency (30-70ms) H.264/H.265 screen capture, audio forward, touch/mouse injection, and clipboard sync. |
| **Rust ADB Client** | [sharpjs/adb_client](https://github.com/sharpjs/adb_client) / `adb_client` (Rust) | MIT | Pure Rust implementation of Android Debug Bridge protocol to discover USB/Wi-Fi devices without relying on external shell scripts. |
| **Rust Scrcpy Client Protocol** | [barry-ran/scrcpy-rs](https://github.com/barry-ran/scrcpy-rs) (Rust) | MIT | Rust bindings that parse the Scrcpy video/audio stream and send mouse/key events. |

### Integration Blueprint in `sricap-rec`:
- **New Tab in GPUI**: Add `apps/desktop-gpui/src/android_window.rs` or an `Android` tab in the main sidebar.
- **Workflow**:
  1. Detect connected devices via `adb_client` or bundled `adb`.
  2. Spawn `scrcpy` server over ADB tunnel (`adb push scrcpy-server.jar /data/local/tmp/`).
  3. Stream video frames directly into a GPUI / Metal canvas view for interactive touch/click/drag manipulation.
  4. Allow one-click screen recording of the Android feed combined with your Mac webcam and microphone.

---

## 2. AI Audio & Disturbance Removal Subsystem (Studio Voice)

### Verified Open-Source Repositories
| Component | Repository & Technology | License | Purpose & Role |
| :--- | :--- | :--- | :--- |
| **Pure Rust RNNoise Engine** | [jneem/nnnoiseless](https://github.com/jneem/nnnoiseless) (Rust) | BSD-3-Clause | 100% safe Rust recurrent neural network (RNN) for real-time speech noise reduction (< 2 MB memory footprint, zero C-compiler dependency). |
| **Deep Learning Speech Denoiser** | [Datadog/dtln-rs](https://github.com/DataDog/dtln-rs) (Rust) | Apache 2.0 | Dual-signal transformation neural network in Rust; state-of-the-art cancellation for background chatter, keyboard clicks, and acoustics. |
| **Dynamic Audio Gain & Gate** | `crates/audio` filter pipeline | Internal | Automatic gain leveling, peak limiting, and noise gate thresholding. |

### Integration Blueprint in `sricap-rec`:
- Wire `nnnoiseless::DenoiseState` into `crates/audio/src/voice.rs`.
- When the user enables the **"Studio Voice / Noise Shield"** toggle:
  - Process raw microphone PCM chunks (48 kHz / 16-bit float) through the RNN filter before encoding into AAC/Opus.
  - Strips air conditioning hum, PC fans, keyboard clicks, and room echo completely.

---

## 3. Offline AI Subtitles & Animated Captions

### Verified Open-Source Repositories
| Component | Repository & Technology | License | Purpose & Role |
| :--- | :--- | :--- | :--- |
| **Rust Whisper Engine** | [tazz4843/whisper-rs](https://github.com/tazz4843/whisper-rs) or [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | MIT | Apple Metal-accelerated offline speech-to-text with word-level timestamps. |
| **ONNX Runtime Speech** | Already bundled in `target/native-deps/onnxruntime` | MIT | Offline transcription using pre-existing embedded ONNX models. |

### Integration Blueprint in `sricap-rec`:
- Single-click **"Generate Captions"** button in `apps/desktop-gpui/src/editor_tabs.rs`.
- Renders dynamic animated word-by-word highlights directly in `crates/rendering/src/layers/captions.rs` and exports `.srt`/`.vtt` subtitle files.

---

## 4. Next-Gen Neural Persona, Eyewear & Gaze Correction

### Verified Open-Source Repositories
| Component | Repository & Technology | License | Purpose & Role |
| :--- | :--- | :--- | :--- |
| **Face Landmark Tracking** | Apple Vision Framework (`VNDetectFaceLandmarksRequest`) | Native macOS | 76-point real-time facial landmark tracking at 60 FPS on Apple Silicon Neural Engine. |
| **Gaze & Eye Contact Alignment** | OpenVINO / ONNX Gaze Correction models | Apache 2.0 | Redirects off-center pupil vectors towards the webcam lens for seamless eye contact. |
| **Neural Mesh & Shading** | Metal Shading Language (`crates/rendering`) | AGPLv3 / Native | Real-time overlays for hair style contouring, glasses, and studio rim lighting. |

---

## 5. Implementation Roadmap Matrix

```
┌────────────────────────────────────────────────────────────────────────┐
│                        sricap-rec Monorepo                             │
├──────────────────┬──────────────────┬──────────────────┬───────────────┤
│    Phase 1       │    Phase 2       │    Phase 3       │    Phase 4    │
│  Studio Voice    │  Android Mirror  │  Auto-Captions   │ Neural Studio │
│ (Noise Stripper) │ & Remote Control │ (Whisper / Metal)│ & Persona FX  │
├──────────────────┼──────────────────┼──────────────────┼───────────────┤
│ • `nnnoiseless`  │ • `scrcpy` core  │ • Word highlight │ • Gaze center │
│ • AC/Fan filter  │ • `adb_client`   │ • .srt / .vtt    │ • Smart glass │
│ • Gain normalize │ • Mouse & Touch  │ • Karaoke styles │ • Hair styles │
│ • VU pulse meter │ • Direct capture │ • Fast export    │ • Metal mesh  │
└──────────────────┴──────────────────┴──────────────────┴───────────────┘
```
