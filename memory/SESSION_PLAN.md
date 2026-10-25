# Argon Speedtest HUD, Free-to-Air Live TV, Studio Pro Video Player & Universal Book Reader Plan

## Step-by-Step Feature Checklist (Tick Boxes Updated as Completed)

### Core Foundations
- [x] **Core 1**: Minimal Top Live Speedometer HUD pill (`[ 🟢 ↓ 342 M  ↑ 88 M ]`) & Network Telemetry Modal in `main_window.rs` & `speedtest.rs`.
- [x] **Core 2**: `"Play With..."` quick-picker modal (`sricap Studio Pro Player` `PRO DEFAULT`, QuickTime, VLC, IINA) & macOS LaunchServices default video player registration (`platform.rs`, `store.rs`, `tauri.conf.json`).
- [x] **Core 3**: Studio Pro Video Player (`player_window.rs`) with Metal 4K hardware decode preview, SMPTE `J`/`L` frame stepper, `0.25x`–`8.0x` pitch-corrected scrubbing, A-B loop engine, EBU R128 LUFS loudness radar, RGB/Luma scopes, `400%` Pixel Loupe, Aspect Ratio safe guides, and Lossless `I`/`O` instant cut.

### Part A: Free-to-Air Global Live TV Engine (10 Novelty Features #1–#10)
- [x] **Feature #1 (Zero-Cost Free-to-Air Directory)**: Native `iptv-org` verified legal public channel directory (NASA TV, Bloomberg TV, DW English, France 24, NHK World, PBS, Euronews, WeatherNation) + custom `.m3u8` stream loader.
- [x] **Feature #2 (Apple TV Style Live EPG)**: 7-day visual Electronic Program Guide showing current and upcoming live broadcasts with progress scrubbers.
- [x] **Feature #3 (Multi-View Sports/News Mosaic 4-Channel Grid)**: Watch up to 4 live free-to-air feeds simultaneously in a synchronized 2×2 grid with 1-click active audio switching.
- [x] **Feature #4 (DVR Catch-Up & 30-Minute Live Pause Buffer)**: In-memory rolling live ring buffer allowing pause, rewind (`-30s`, `-5m`, `-30m`), and frame-stepping on live TV streams.
- [x] **Feature #5 (Smart Auto-Fallback Stream Resolver)**: Automatic mirror CDN failover (`Primary CDN` → `Mirror CDN`) with zero playback interruption.
- [x] **Feature #6 (Hardware-Decoded Low-Latency HLS/DASH Engine)**: Sub-second channel tuning (`0.28s`) utilizing macOS VideoToolbox zero-copy hardware acceleration.
- [x] **Feature #7 (Picture-in-Picture Floating Glass HUD)**: Always-on-top compact PiP floating HUD mode for watching live TV or tutorials while coding.
- [x] **Feature #8 (Real-Time Live Closed-Captions & Subtitle Translator)**: On-the-fly Whisper live closed-captions with instant subtitle translation (`EN`, `ES`, `DE`, `FR`, `JA`, `HI`).
- [x] **Feature #9 (Instant Stream Clip to Studio Editor `Cmd+Shift+R`)**: 1-click capture of the last 30 seconds of any live stream directly into `<recordings_dir>/Exports/` and the Cap Studio Editor timeline.
- [x] **Feature #10 (Audio-Only Background Radio Mode)**: 1-click `~64 kbps` Audio-Only Radio Mode for listening to global news, tech panels, and music streams with minimal bandwidth.

### Part B: Pro Internet Radar & Network Novelty Suite (5 Novelty Features #11–#15)
- [x] **Feature #11 (Wi-Fi 6E/7 BSSID Radar & Spectrum Analyzer)**: Real-time channel congestion, BSSID readout, RSSI signal drop warnings, and 2.4/5/6 GHz interference telemetry.
- [x] **Feature #12 (Streaming Bufferbloat Grade & QoS Optimizer)**: Real-time packet queuing latency evaluation under load (`A+` through `F` grade) + 1-click QoS buffer pacing optimizer.
- [x] **Feature #13 (CDN Edge Latency Map)**: Multi-POP latency probe across Cloudflare (`1.1.1.1`), AWS CloudFront, Fastly, and Akamai with optimal upload region recommendation.
- [x] **Feature #14 (Packet Loss Forensics & Jitter Waveform)**: Live rolling oscilloscope waveform of network jitter and packet stability in the Network Telemetry modal.
- [x] **Feature #15 (Auto-Throttle Bandwidth Saver)**: Toggleable metered-connection / low-battery auto-throttle bandwidth saver shared between Network Radar and Player.

### Part C: Studio Pro Video Player Novelty Tools (5 Novelty Features #16–#20)
- [x] **Feature #16 (AI Speech Clarity & Dialog Boost)**: Multi-band vocal enhancer (`+4.5 dB` 2–5 kHz presence boost + muddy background attenuation).
- [x] **Feature #17 (Dynamic Volume Night Mode)**: `-16 LUFS` soft-knee transient compressor that tames loud explosions and spikes while keeping dialogue clear.
- [x] **Feature #18 (Precision Subtitle Resync Waveform)**: Visual subtitle waveform sync controls (`Sub -50ms` / `Sub +50ms`).
- [x] **Feature #19 (Lossless Frame Extraction & Color Sampler)**: 10-bit / 4K uncompressed PNG frame extraction + live Hex / RGB / Rec.709 color sampler readout.
- [x] **Feature #20 (macOS Spatial Audio & Head-Tracking Support)**: Multi-channel spatial soundstage mode (`Reference Stereo`, `Spatial Binaural`, `Spatial 7.1.4 Head-Tracked` for AirPods Pro/Max).

### Section 3: Universal $200 Tier Book & Document Reader Studio (`reader_window.rs`)
- [x] **Reader #1 (Formats & Header Trigger)**: Dedicated `"open-book-reader"` button (`icons/book-open.svg`, 16px) in `main_window.rs` header bar + `ReaderWindow` supporting PDF, EPUB, MOBI, AZW3, DJVU, CBZ/CBR, and Markdown (`.md`).
- [x] **Reader #2 (Typography, Themes & Bionic Reading Engine)**: `Warm Paper`, `E-Ink Monochrome`, `Dark Obsidian`, and `Solarized Sepia` themes; `Serif`, `Sans`, and `Monospace` fonts; **Bionic Reading Toggle**; and Apple Neural TTS (`0.5x`–`3.0x`).
- [x] **Reader #3 (Smart Learning, Annotations & AI Study Companion)**: Color-coded highlights (`Yellow: Core Concept`, `Purple: Quote`, `Red: Counterargument`), sticky notes, TOC sidebar, search & bookmarks, Two-Page Spread toggle, Zen Fullscreen mode, 1-click `"Synthesize Chapter Summary"`, and 1-click `"Export Markdown Notes & AI Flashcards"`.
- [x] **Verification & Release Changelog**: Update `CHANGELOG.md`, `settings_pages.rs`, and `memory/ARGON_SPEEDTEST_AND_PLAYER_PROMPT.md` with `[x]` tick boxes and run `cargo fmt --all` and `cargo test --bin sricap-rec`.
