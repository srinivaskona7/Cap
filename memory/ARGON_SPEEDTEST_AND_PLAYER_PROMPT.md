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

## 🛠 SECTION 4: Technical Implementation Specifications

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

---

## 📋 Copyable Master Prompt for Argon

```text
Please implement the following three studio-grade features in sricap-rec (apps/desktop-gpui):

1. Minimal Top Live Speedometer HUD & Telemetry Modal:
- In apps/desktop-gpui/src/main_window.rs header, add a minimalist glassmorphism pill displaying live speeds: "[ 🟢 ↓ 342 M  ↑ 88 M ]".
- Hovering shows: "Network Health: Excellent (11ms) • Click for Full Network Radar & Diagnostics".
- Clicking this pill opens a high-fidelity "Network & Streaming Telemetry" modal featuring:
  • Real-time animated Download / Upload Mbps speed gauge
  • Ping (RTT), Jitter (ms), and Packet Loss (0.0%)
  • Streaming readiness ratings (4K 60fps cloud upload, RTMP Twitch/YouTube headroom)
  • Wi-Fi PHY channel details (5GHz/6GHz, RSSI, channel width, Bufferbloat Grade A+)
  • 1-Click "Copy Diagnostics Report (.MD)" button

2. Dedicated Studio Pro Video Player & "Play With..." Suite:
- Add a dedicated "Studio Pro Video Player" button (icon: play-square.svg, 16px) to the main_window.rs header bar.
- Implement a "Play With..." context modal when double-clicking videos or selecting play options, showing:
  • sricap Studio Pro Player (Metal 4K, Frame Stepper, LUFS Radar, Scopes, Lossless Cut) marked as Recommended Pro Default
  • QuickTime Player, VLC, and IINA detected app options
  • "[✓] Always open with sricap Studio Player" checkbox
  • "Set sricap as macOS Default Video Player" system button via LaunchServices
- Implement the Studio Pro Video Player in apps/desktop-gpui/src/player_window.rs with:
  • Zero-copy Metal 4K 60fps hardware decoding
  • SMPTE Frame-by-frame stepper (J/L keys)
  • EBU R128 LUFS live loudness radar with YouTube -14 LUFS guide
  • RGB Parade & Luminance Waveform scopes
  • Floating 400% pixel magnifier loupe
  • Instant lossless In/Out trimmer (I/O keys) with sub-second remux export

3. Universal Book & Document Reader Studio:
- Add a dedicated "Universal Book Reader" button (icon: book-open.svg, 16px) to the main_window.rs header bar.
- Implement apps/desktop-gpui/src/reader_window.rs supporting:
  • Formats: PDF, EPUB, MOBI, CBZ, and Markdown (.md)
  • Themes: Warm Paper, E-Ink Monochrome, Dark Obsidian, and Solarized Sepia
  • Bionic Reading Mode toggle (guided fixation typography)
  • Interactive TOC sidebar, bookmarks, and color-coded annotation highlighter
  • 1-Click "Export Markdown Notes & AI Flashcards" compatible with BOOK_LEARNING_PROTOCOL.md

Adhere strictly to repo invariants: Zero unnecessary comments, tab indentation where applicable, Clippy-clean Rust, strict 8-point layout rhythm, and no overlapping hit targets.
```
