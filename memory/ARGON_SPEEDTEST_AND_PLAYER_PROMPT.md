# Argon UI/UX Engineering Prompt: Top Live Speedometer HUD & $200 Tier Video Player Suite

---

## 🎯 Executive Overview & Mission
You are tasked with implementing two flagship, studio-grade features into **`sricap-rec`** (`apps/desktop-gpui`):
1. **Ultra-Minimal Header Live Internet Speedometer & Full-Spectrum Network Telemetry Hub**:
   - A featherweight, minimalist pill in the top header displaying real-time live download/upload speeds (`↓ 340 Mbps  ↑ 85 Mbps`).
   - Clicking the pill opens an interactive **Network & Streaming Telemetry HUD** providing enterprise-grade analytics (packet jitter, buffer bloat, CDN edge latency, live streaming bitrate health, and Wi-Fi PHY frequency metrics) entirely free.
2. **Dedicated Studio Pro Video Player & "Play With..." Ecosystem**:
   - A dedicated **Studio Video Player** icon button in the main header and library cards.
   - A full-featured modal dialog asking **"Play With..."** allowing users to launch files with `sricap Studio Player` (Metal zero-copy, frame stepper, LUFS meter, color vectorscope, lossless trimmer) or system players (QuickTime, VLC, IINA).
   - A 1-click **"Set sricap as Default macOS Video Player"** integration.

---

## 📐 SECTION 1: Minimal Top Speedometer Pill & Deep Diagnostics HUD

### 1. Header Layout & Placement
In `apps/desktop-gpui/src/main_window.rs` (inside the header `actions` bar next to the existing utility buttons):
- **Visual Footprint**: Ultra-compact glassmorphism pill (`h = 24px`, `px = 8px`, rounded full, border `theme.border_subtle`, background `rgba(255, 255, 255, 0.04)`).
- **Default Display Syntax**:
  ```
  [ 🟢 ↓ 342 M  ↑ 88 M ]
  ```
  *(Dynamic auto-formatting: `Kbps` if < 1 Mbps, `Mbps` if < 1000 Mbps, `Gbps` if >= 1000 Mbps)*
- **Hover Micro-Interaction**:
  - Background darkens to `rgba(255, 255, 255, 0.08)`.
  - Tooltip: `"Network Health: Excellent (11ms) • Click for Full Network Radar & Diagnostics"`.

### 2. Deep Telemetry Diagnostics Popover / Modal (Triggered on Click)
Clicking the speedometer pill opens the **Network Radar & Broadcast Diagnostics Panel**:
- **Core Speed Burst Gauge**:
  - Real-time animated speed graph with Ping (RTT), Jitter (± ms), Packet Loss (`0.0%`), Download Mbps, and Upload Mbps.
- **Streaming & Recording Safety Evaluator**:
  - Live indicator:
    - `4K 60fps Cloud Upload: 100% Ready (Zero Dropped Frames)`
    - `YouTube / Twitch RTMP Ingest: Stable (12,000 kbps headroom)`
    - `Multi-Cam Remote Peer Streaming: Sub-15ms Latency`
- **Network Interface & ISP Telemetry**:
  - Connection type: `Wi-Fi 6E (802.11ax) • 5 GHz • 160 MHz Channel • RSSI: -48 dBm` or `Thunderbolt 10GbE`.
  - Public & Local IP, Gateway Ping, DNS Resolver Speed (Cloudflare vs Google DNS comparison), Bufferbloat Grade (`A+ / Low latency under load`).
- **One-Click Actions**:
  - `[ ⚡ Re-Run Complete Diagnostics ]`
  - `[ 📋 Copy Diagnostics Report for IT / Support (.MD) ]`
  - `[ 🛠 Optimize Network Buffer for Live Recording ]`

---

## 🎬 SECTION 2: $200 Tier Studio Pro Video Player Suite

### 1. Dedicated Video Player Header Button
Add a dedicated icon button in `main_window.rs` header (next to `open-studio-direct` and `recordings`):
- **Button ID**: `"open-video-player"`
- **Icon**: `"icons/play-square.svg"` or custom video screen glyph (`16px`).
- **Tooltip**: `"Studio Pro Video Player (Metal 4K · Frame Stepper · LUFS Radar)"`.
- **Action**: Opens file picker or the latest recorded/imported clip directly in the **Player Window**.

### 2. "Play With..." Context Quick-Picker Modal
When the user double-clicks any clip in the library or clicks the playback trigger:
- **Modal Header**: Display filename, resolution, framerate, duration, and audio codec.
- **Options List**:
  1. **⚡ sricap Studio Pro Player (Recommended - Free)**
     - Subtext: *Metal Hardware Accelerated • Frame-by-Frame Stepper • EBU R128 Loudness Radar • Color Scopes • Lossless Cut*
     - Badge: `PRO DEFAULT` (Neon accent pill).
  2. **🎬 QuickTime Player**
  3. **🍊 VLC Media Player** (if detected in `/Applications`)
  4. **✨ IINA / Movist Pro** (if detected in `/Applications`)
- **Footer Toggles**:
  - Checkbox: `[✓] Always open with sricap Studio Player`
  - Button: `[ 🌐 Make sricap Default Video Player for macOS ]`

### 3. $200 Premium Player Capabilities (Inside `player_window.rs`)
The player must feature a clean, floating, distraction-free Dark Obsidian UI:

#### A. Playback & Scrubbing Precision
- **Zero-Copy Metal Pipeline**: Smooth 4K/8K 60fps playback with zero frame stuttering.
- **Pro Frame Stepper**: Frame backwards (`J` / `←`) and forward (`L` / `→`) with exact SMPTE timecode (`00:04:12:28`).
- **Pitch-Corrected Scrubbing**: `0.25x`, `0.5x`, `1.0x`, `1.25x`, `1.5x`, `2.0x`, `4.0x`, `8.0x` with instant audio resync.
- **A-B Loop Engine**: Mark point `A` and point `B` to loop specific tutorial segments or UI bugs endlessly.

#### B. Studio Audio & Loudness Metering
- **Real-Time EBU R128 LUFS Radar**: Live loudness meter displaying Integrated LUFS, Short-Term LUFS, and True Peak dBTP.
- **YouTube/TikTok/Podcast Target Guide**: Visual indicators showing if audio is at `-14 LUFS` (YouTube standard) or `-16 LUFS` (Apple Podcasts).
- **Audio Stem Switcher**: Instant toggle to isolate Mic, Desktop Audio, or System Sound independently.

#### C. Visual Inspection & Color Scopes
- **Luminance Waveform & RGB Parade**: Live scopes for highlight clipping and shadow compression analysis.
- **Pixel Magnifier Loupe**: Floating 400% zoom lens to inspect UI font anti-aliasing, icon sharpness, and compression artifacts.
- **Aspect Ratio Guides**: 16:9 YouTube, 9:16 Shorts/Reels, 1:1 Square, 4:5 Instagram safe zone overlays.

#### D. Lossless Instant Cut & Snapshot Export
- **Lossless In/Out Trimmer (`I` / `O`)**: Export trimmed sections in < 0.5 seconds without re-encoding (smart stream remux via `cap-muxer`).
- **Lossless Frame Grabber**: Capture the exact frame as a 4K PNG/WebP with metadata stamp.

---

## 🛠 SECTION 3: Technical Implementation Specifications

### 1. Speed Test Engine (`apps/desktop-gpui/src/speedtest.rs`)
```rust
#[derive(Clone, Debug, Default)]
pub struct SpeedTestResult {
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub ping_ms: u32,
    pub jitter_ms: u32,
    pub packet_loss_pct: f32,
    pub wifi_standard: String,
    pub channel_freq: String,
    pub bufferbloat_grade: String,
    pub is_testing: bool,
}
```
- Implement lightweight asynchronous socket burst using standard Tokio tasks.
- Keep background bandwidth negligible (< 15KB per minute during idle heartbeat).

### 2. macOS Default Player Association
- In `apps/desktop-gpui/src/platform.rs`:
  ```rust
  #[cfg(target_os = "macos")]
  pub fn set_as_default_video_player() -> Result<(), String> {
      // Calls macOS LaunchServices API:
      // LSSetDefaultRoleHandlerForContentType("public.movie" as CFString, kLSRolesViewer, bundle_id)
      // LSSetDefaultRoleHandlerForContentType("public.mpeg-4" as CFString, kLSRolesViewer, bundle_id)
  }
  ```
- Support `.mp4`, `.mov`, `.mkv`, `.webm`, `.m4v`, `.cap`.

## 📚 SECTION 3: Universal $200 Tier Book & Document Reader Studio

### 1. Formats Supported
- **Documents & Textbooks**: PDF, EPUB, MOBI, AZW3, DJVU.
- **Developer & Technical**: Markdown (`.md`), HTML, AsciiDoc, Org-Mode.
- **Comics & Manga**: CBZ, CBR.

### 2. Dedicated Reader Header Button & Quick Trigger
Add a dedicated icon button in `main_window.rs` header:
- **Button ID**: `"open-book-reader"`
- **Icon**: `"icons/book-open.svg"` (16px).
- **Tooltip**: `"Universal Studio Book Reader (PDF · EPUB · Markdown · AI Neural Flashcards)"`.
- **Action**: Opens file picker or the Book Learning Vault (`reader_window.rs`).

### 3. $200 Premium Book Reader Capabilities (Inside `reader_window.rs`)
- **Typography & Bionic Reading Engine**:
  - Warm Paper, E-Ink Monochrome, Dark Obsidian, and Solarized Sepia themes.
  - Serif (Charter/Merriweather), Sans (Inter/SF Pro), and Monospace (JetBrains Mono) font modes.
  - **Bionic Reading Toggle**: Highlights the initial letters of each word to boost reading speed by 40%.
  - Text-to-Speech Voice Engine (Apple Neural Voice with speed control 0.5x–3.0x).
- **Smart Learning & Annotation Hub**:
  - Color-coded highlights (Yellow: Core Concept, Purple: Quote, Red: Objection/Counterargument).
  - Margin sticky notes with instant Markdown export.
  - Integrated Table of Contents (TOC) sidebar with search indexing and page bookmarks.
- **AI Deep Study Companion Integration**:
  - 1-Click **"Synthesize Chapter Summary"** into structured key takeaways.
  - 1-Click **"Generate Retrieval Flashcards"** (integrated directly into the Book Learning Protocol).
  - Inline Terminology Inspector: Hover over complex terms for contextual definitions.
- **Two-Page Spread & Presentation Mode**:
  - Continuous vertical scroll or Kindle-style two-page horizontal flip.
  - Distraction-Free Zen Fullscreen (auto-hides macOS dock and menubar).

---

## 🌟 SECTION 4: 20 Novelty & Free-to-Air Live TV Features (Compete with $200 Apps)

Drawn from leading open-source broadcast repositories (`iptv-org/iptv`, `Free-TV/IPTV`, `Streamlink`, `IINA+`, and `mpv`):

### A. Free-to-Air Live TV & Global Broadcast Engine (10 Novelty Features)
- [x] **1. Zero-Cost Global Free-to-Air TV Directory**: Direct integration with the verified `iptv-org` legal public broadcast index (over 8,000+ public news, NASA TV, Bloomberg, DW, France24, NHK World, PBS, weather, and educational channels streaming 100% free with zero subscriptions).
- [x] **2. Apple TV Style Live EPG (Electronic Program Guide)**: Clean 7-day visual timeline grid showing current and upcoming shows with live progress bars.
- [x] **3. Multi-View Sports & News Mosaic (4-Channel Grid)**: Watch up to 4 live free-to-air feeds simultaneously in a synchronized 2×2 grid with 1-click audio switching.
- [x] **4. DVR Catch-Up & 30-Minute Live Pause Buffer**: Pause, rewind, or slow-motion scrub live broadcast streams using an in-memory rolling ring buffer.
- [x] **5. Smart Auto-Fallback Stream Resolver**: If a public CDN link drops or lags, automatically rotates to the fastest verified mirror stream without interrupting playback.
- [x] **6. Hardware-Decoded Low-Latency HLS/DASH Engine**: Instant sub-second channel tuning utilizing Apple VideoToolbox hardware acceleration.
- [x] **7. Picture-in-Picture (PiP) Floating HUD**: Detach any live TV broadcast or tutorial video into an always-on-top borderless floating glass window while coding.
- [x] **8. Real-Time Live Closed-Captions & Subtitle Translator**: On-the-fly local whisper transcription and subtitle translation for foreign public broadcasts.
- [x] **9. Instant Stream Clip to Studio Editor**: 1-click `Cmd+Shift+R` to grab a 30s highlight from any live free-to-air stream directly into the Cap Studio editor for reels/shorts creation.
- [x] **10. Audio-Only Background Radio Mode**: Minimize bandwidth to ~64 kbps to listen to global news, tech panels, and music streams with screen off.

### B. Pro Internet Radar & Network Novelty Suite (5 Novelty Features)
- [x] **11. Wi-Fi 6E/7 BSSID Radar & Spectrum Analyzer**: Displays real-time channel congestion, RSSI signal drop warnings, and frequency interference.
- [x] **12. Streaming Bufferbloat Grade & QoS Optimizer**: Evaluates real-time packet queuing latency under heavy network load with an A+ through F grading scale.
- [x] **13. CDN Edge Latency Map**: Pings Cloudflare, AWS CloudFront, Fastly, and Akamai POPs to recommend the optimal upload region.
- [x] **14. Packet Loss Forensics & Jitter Waveform**: Displays a live rolling oscilloscope graph of network stability during live broadcasts.
- [x] **15. Auto-Throttle Bandwidth Saver**: Automatically downscales background playback when metering connections or running on low battery.

### C. Studio Pro Video Player Novelty Tools (5 Novelty Features)
- [x] **16. AI Speech Clarity & Dialog Boost**: Multi-band speech enhancer that isolates vocals and silences muddy background music in movies or noisy tutorials.
- [x] **17. Dynamic Volume Night Mode**: Smooths out explosions and abrupt loud sounds to maintain comfortable, consistent dialogue volume.
- [x] **18. Precision Subtitle Resync Waveform**: Visual timeline where users can drag subtitle tracks against audio peaks for millisecond-perfect lip sync.
- [x] **19. Lossless Frame Extraction & Color Sampler**: Extract uncompressed 10-bit PNG stills and inspect hex/RGB values directly from video frames.
- [x] **20. macOS Spatial Audio & Head-Tracking Support**: Immersive multi-channel sound virtualization for AirPods Pro/Max users.

### 1. Speed Test Engine (`apps/desktop-gpui/src/speedtest.rs`)
```rust
#[derive(Clone, Debug, Default)]
pub struct SpeedTestResult {
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub ping_ms: u32,
    pub jitter_ms: u32,
    pub packet_loss_pct: f32,
    pub wifi_standard: String,
    pub channel_freq: String,
    pub bufferbloat_grade: String,
    pub is_testing: bool,
}
```

### 2. Universal Reader Engine (`apps/desktop-gpui/src/reader_window.rs`)
- High-performance PDF rendering using `pdfium` / CoreGraphics bindings on macOS.
- EPUB parser extracting HTML/CSS spine with GPUI rich text layout.
- Markdown viewer with live Mermaid diagram rendering.

### 3. Strict Pre-Generation Invariants for Code Generation
Argon must follow all invariants in [AGENTS.md](file:///Users/sr20536224wipro.com/Documents/Github-apple/srinew/Cap/AGENTS.md):
- **Zero code comments**: Only add comments for non-obvious platform workarounds.
- **Strict 8-Point Rhythm**: Use `4px`, `8px`, `12px`, `16px`, `24px` for all gaps, paddings, and bounds.
- **No Text Overlap**: Hit targets must be at least `32×32pt` in GPUI.
- **Clippy-Clean Rust**: Deny `let_underscore_future`, deny `dbg!`, use `.clamp()`, use `saturating_sub()`.

## 🎙️ SECTION 5: Transcription Model Hub (Primary, Secondary/Fallback & Clean Purge)

### 1. UI Model Cards & Status Badges
Inside `apps/desktop-gpui/src/editor_tabs.rs` (Captions Tab) and `settings_pages.rs`:
- Every model is displayed as an interactive card with:
  - **Model Title & Architecture**: e.g., `NVIDIA Parakeet TDT (INT8)` or `OpenAI Whisper Medium`.
  - **Footprint & State**: `Downloaded (670 MB)` or `Not Downloaded (Click to Install)`.
  - **Role Selector**: Radio/dropdown toggles:
    - `Primary Engine`: (First choice for auto-captioning).
    - `Secondary / Fallback Engine`: (Used automatically if primary is busy or audio language is not supported).
  - **Permanent Delete / Clean Button**:
    - Purges the model directory (`rm -rf ~/Library/Application Support/so.cap.desktop/transcription_models/<model>`) freeing disk space immediately.
  - **1-Click Install Button**:
    - Downloads and verifies ONNX/GGML chunks with real-time percentage progress bar.

---

## ⚡ SECTION 6: Reverse-Engineered Parakeet-AI Call & Mock Interview Copilot

Reverse-engineered from analysis of `parakeet-ai.com`:

### 1. Core Architecture & Secret Free Fast APIs
- **Zero-Cloud Audio Streaming**: Your Mac's local **NVIDIA Parakeet TDT** transcribes live speaker and microphone audio in < 200ms latency.
- **Pluggable Fast LLM Calling**: Configurable directly from your `.env` without recurring subscriptions:
  - Free/Low-latency providers:
    - **Groq Cloud API** (`GROQ_API_KEY`: LLaMA-3.3 70B streaming at ~300 tokens/sec — completely free tier available).
    - **Google Gemini Flash API** (`GEMINI_API_KEY`: gemini-1.5-flash / 2.0-flash with sub-500ms time-to-first-token).
    - **Ollama Local Endpoint** (`OLLAMA_HOST=http://localhost:11434`: 100% offline, zero internet required).

### 2. Standalone Real-Time Floating HUD (`apps/desktop-gpui/src/copilot_window.rs`)
- **Undetectable Screen-Share Stealth Mode**: Uses macOS CoreGraphics window level flags (`kCGBackingStoreNonretained` + `sharingState = none`) so Zoom, Google Meet, Teams, and Webex screen-sharing **never capture the copilot window**.
- **Real-Time Question Interceptor**: Automatically detects interviewer questions (technical algorithms, behavioral STAR questions, system design).
- **Custom Context Injection**: Upload or paste your Resume/CV and the Job Description (`JD`). The AI streams tailored, first-person talking points instantly as the interviewer speaks.
- **Interactive Mock Interview Simulator (`/mock-interview`)**:
  - Practice real rounds with an AI interviewer asking follow-ups and grading technical depth and communication pacing.

---

## 📋 Copyable Master Prompt for Argon

```text
Please implement the following five studio-grade features in sricap-rec (apps/desktop-gpui):

1. Minimal Top Live Speedometer HUD & Telemetry Modal:
- In apps/desktop-gpui/src/main_window.rs header, add a minimalist glassmorphism pill: "[ 🟢 ↓ 342 M  ↑ 88 M ]".
- Clicking opens a Network Radar modal with Jitter, Ping, Bufferbloat Grade A+, Wi-Fi 6E PHY metrics, and 1-click Markdown diagnostics export.

2. Dedicated Studio Pro Video Player & "Play With..." Suite:
- Add a dedicated "Studio Pro Video Player" button to main_window.rs header.
- Implement "Play With..." context modal with sricap Studio Player, QuickTime, VLC, IINA, and "Set as Default macOS Video Player" LaunchServices button.
- Implement player_window.rs with Metal 4K 60fps, frame stepper (J/L), EBU R128 LUFS radar, RGB scopes, 400% loupe, and lossless I/O cut.

3. Universal Book & Document Reader Studio:
- Add "Universal Book Reader" button (PDF, EPUB, MOBI, CBZ, Markdown .md) to header.
- Implement reader_window.rs with Warm Paper, E-Ink, Dark Obsidian, Bionic Reading Mode, TOC sidebar, and AI Flashcard/SOP export.

4. Advanced Transcription Model Hub & Cleaner:
- In editor_tabs.rs (Captions tab), list all models (Parakeet INT8, Parakeet Max, Whisper Small, Whisper Medium) with exact file sizes.
- Add "Primary Engine" and "Secondary / Fallback Engine" toggles.
- Add 1-click "Download / Install" and "Delete Permanently" buttons with instant disk cleanup.

5. Parakeet-AI Real-Time Call & Mock Interview Copilot:
- Implement a dedicated "AI Call & Interview Copilot" mode (undetectable screen-share stealth overlay).
- Uses local Parakeet TDT for <200ms speech-to-text.
- Calls fast streaming LLM APIs (configured via .env: GROQ_API_KEY, GEMINI_API_KEY, or local Ollama).
- Provides live question interception, resume/JD context injection, and an interactive mock interview practice mode.

Adhere strictly to repo invariants: Zero unnecessary comments, tab indentation where applicable, Clippy-clean Rust, strict 8-point layout rhythm, and no overlapping hit targets.
```
