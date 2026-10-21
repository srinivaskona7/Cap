# Rust Media Engine (`crates/*`)

The `crates/` directory contains 50+ specialized Rust crates that power screen capture, camera/audio acquisition, GPU compositing, hardware encoding/decoding, crash-safe muxing, and automated A/V verification.

## 1. Recording & Project Lifecycle

- **`crates/recording` (`cap-recording`)**:
  - Actor-driven (`kameo`) recording orchestrator.
  - **`instant_recording`**: Real-time capture → resize/convert → encode → fragmented/progressive upload (`upload_preparation`, `upload_resume`, `upload_verification`).
  - **`studio_recording`**: Captures multi-segment high-quality display + camera + mic + system audio tracks + cursor/keyboard telemetry (`cursor.rs`) for non-linear editing.
  - **Resilience**: Includes `recovery.rs` (recovering incomplete recordings), `track_heal.rs`, `sync_calibration.rs`, and `oop_muxer` (out-of-process muxing).
- **`crates/project` (`cap-project`)**:
  - Data structures for `.cap` project directories: `ProjectConfiguration`, `RecordingMeta`, `StudioRecordingMeta`, timeline segments, `ClipOffsets`, `CursorEvents`, zoom segments, captions, keyboard overlays, and camera/frame layout config.
- **`crates/editor` (`cap-editor`)**:
  - Framework-agnostic editor playback & scrubbing engine (`EditorInstance`).
  - Owns async video decoders, audio playback (`rodio`), the renderer actor, and scrub preview state, returning rendered frames via callbacks (used by both `apps/desktop-gpui` and `apps/desktop`).
- **`crates/export` (`cap-export`)**:
  - Drives full-project rendering to MP4 (`enc-ffmpeg` / hardware encoders) or GIF (`enc-gif`).

---

## 2. Screen & Cursor Capture (`scap-*`, `cursor-*`)

- **`crates/scap-targets`**: Cross-platform enumeration and geometry (`LogicalBounds`, `PhysicalSize`) for displays, windows, and crop areas.
- **`crates/scap-screencapturekit`** *(macOS)*: Built on `cidre` bindings to Apple's `ScreenCaptureKit`. Delivers zero-copy `CVPixelBuffer` / `IOSurface` frames and macOS system audio.
- **`crates/scap-direct3d`** *(Windows)*: `Windows.Graphics.Capture` + Direct3D 11 (`crates/d3d-adapter`) hardware texture capture.
- **`crates/scap-ffmpeg`**: FFmpeg/PipeWire/X11 screen capture fallback.
- **`crates/cursor-capture` & `crates/cursor-info`**: Polls cursor coordinates, click states, and system cursor shapes, mapping them to vector SVG cursor assets.

---

## 3. Camera & Audio Pipeline

- **Camera (`crates/camera*`)**:
  - `camera`: Unified cross-platform camera abstraction.
  - Platform backends: `camera-avfoundation` (macOS), `camera-mediafoundation` & `camera-directshow` (Windows), `camera-windows`, `camera-ffmpeg`.
  - `camera-effects`: GPU-driven portrait segmentation and background blur (`BlurProcessor`, `wgpu` 25).
- **Audio (`crates/audio`, `crates/scap-cpal`, `crates/cpal-ffmpeg`, `crates/rnnoise`)**:
  - Uses patched `CapSoftware/cpal` (fixes macOS CoreAudio drop lifecycle and zeros Windows WASAPI `AUDCLNT_BUFFERFLAGS_SILENT` loopback packets).
  - Real-time neural noise suppression via `crates/rnnoise`.

---

## 4. GPU Rendering & Compositing (`crates/rendering*`)

- **`crates/rendering` (`cap-rendering`)**:
  - `wgpu` v25 compositor that renders each video frame through modular shader layers (`src/layers/`):
    - `BackgroundLayer` (wallpapers, gradients, solid colors, blur)
    - `DisplayLayer` & `FrameLayer` (rounded corners, shadows, border chrome, macOS notch `NotchLayer`)
    - `CursorLayer` & `ClickRippleLayer` (spring-mass-damper smoothed cursor trajectory `spring_mass_damper.rs`, motion blur, SVG cursor rasterization)
    - `CameraLayer` & `Camera3DLayer` (webcam bubble, 3D perspective tilt, shadow, shape)
    - `MaskLayer`, `BlurLayer`, `ColorGradeLayer`, `OverlayLayers` (text, captions, keyboard press badges)
    - `zoom.rs` & `zoom_spring.rs`: Automatic and manual smooth zoom/pan interpolation.
  - **Zero-Copy & Fast Readback**:
    - macOS: `iosurface_texture.rs`, `Nv12Surface`, `RgbaToBgraSurfaceConverter` for direct `CVPixelBuffer` handoff to GPUI or VideoToolbox.
    - Windows: `d3d_texture.rs`.
    - Linux: `linux_gpu.rs`.
    - Pooled readback via `NV12BufferPool` (`frame_pipeline.rs`).
- **`crates/rendering-skia`**: Skia 2D rendering backend.
- **`crates/gpu-converters` & `crates/frame-converter`**: Compute/render shaders and FFmpeg swscale wrappers for pixel format conversion (RGBA, BGRA, NV12, YUV420P).

---

## 5. Encoding, Decoding & Crash-Safe Muxing

- **Encoders**:
  - `crates/enc-avfoundation`: macOS VideoToolbox hardware H.264/HEVC encoding.
  - `crates/enc-mediafoundation`: Windows MediaFoundation hardware encoding (`crates/mediafoundation-ffmpeg`, `crates/mediafoundation-utils`).
  - `crates/enc-ffmpeg`: Cross-platform FFmpeg encoding (H.264, AAC, Opus).
  - `crates/enc-gif`: GIF encoding for short exports.
- **Out-of-Process Muxer (`crates/cap-muxer` & `crates/cap-muxer-protocol`)**:
  - Runs MP4/fMP4 muxing in a child process over an IPC protocol so that if the main desktop UI or encoder crashes, the recorded media file can still finalize cleanly.
- **Decoding (`crates/video-decode` & `crates/ffmpeg-hw-device`)**:
  - Hardware-accelerated video decoding (VideoToolbox, D3D11VA/DXVA2, VAAPI) with software fallback for low-latency editor timeline scrubbing.

---

## 6. Deterministic Verification (`crates/verify*`, `crates/cap-test`)

- **`crates/verify-stimulus`**: Generates synthetic visual frames and audio tones encoded with binary frame markers/timestamps.
- **`crates/verify` & `crates/verify-expect`**: Decodes recorded/exported output files, reads embedded pixel/audio markers (compiled at `opt-level = 3` even in dev), and asserts frame delivery, monotonic pacing, A/V synchronization, and color fidelity.
- **`crates/cap-test`**: CLI runner (`bun run test:matrix`, `test:synthetic`, `test:suite:recording`, etc.) for hardware/software recording matrix tests.
