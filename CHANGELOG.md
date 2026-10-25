# Changelog — `sricap-rec` Custom Studio & Screen Recording Suite

All notable features, presets, UI options, and architectural customizations built into **`sricap-rec`** (`com.sri.sricap-rec`) on the `feature1` branch. Every release includes its dedicated changelog section below and in the in-app **Settings → Changelog** viewer.

---

## [v0.6.8] — 2026-10-24

### 1. Minimal Top Live Speedometer HUD & Network Radar Telemetry Modal (`speedtest.rs` & `main_window.rs`)
- **Minimal Top Header Speedometer Pill (`[ 🟢 ↓ 342 M  ↑ 88 M ]`)**:
  - Ultra-compact glassmorphism pill (`22px` height) in the Main Window header bar with dynamic rate auto-formatting (`K` for Kbps, `M` for Mbps, `G` for Gbps) and color-coded health status (`🟢` Excellent, `🟡` Good, `🔴` Degraded).
  - Hovering displays: `"Network Health: Excellent (11ms) • Click for Full Network Radar & Diagnostics"`.
- **Full Network & Streaming Telemetry Modal (`render_speedtest_telemetry_modal`)**:
  - Real-time Download & Upload Mbps burst readout, Ping (RTT ms), Jitter (`±ms`), and Packet Loss (`0.0%`).
  - **Streaming & 4K Cloud Readiness Evaluator**: Evaluates `4K 60fps Cloud Upload`, `YouTube / Twitch RTMP Ingest` headroom, and `Multi-Cam Remote Peer Streaming` latency.
  - **Wi-Fi PHY & Interface Telemetry**: Displays Wi-Fi 6E (`802.11ax`) channel frequency, channel width (`160 MHz`), RSSI (`-48 dBm`), Local IP, Gateway RTT, Cloudflare `1.1.1.1` vs Google `8.8.8.8` DNS speed, and Bufferbloat Grade (`A+`).
  - **1-Click Actions**: **`⚡ Re-Run Complete Diagnostics`**, **`📋 Copy Diagnostics Report (.MD)`**, and **`🛠 Optimize Network Buffer for Live Recording`**.

### 2. Dedicated Studio Pro Video Player & `"Play With..."` Suite (`player_window.rs`, `platform.rs`, & `main_window.rs`)
- **Studio Pro Video Player (`apps/desktop-gpui/src/player_window.rs`)**:
  - **Zero-Copy Metal 4K 60fps Pipeline**: Dedicated dark obsidian playback studio (`open-video-player` button in header bar + play action on every recording card).
  - **Pro SMPTE Frame Stepper & Scrubbing**: Frame-by-frame stepping (`J` / `←` for `-1f`, `L` / `→` for `+1f`, `Space` / `K` for Play/Pause) with exact **SMPTE Timecode (`00:00:00:00`)**, pitch-corrected speeds (`0.25x` through `8.0x`), and **A-B Loop Engine** (`Loop A`, `Loop B`, `Clear Loop`).
  - **Real-Time EBU R128 LUFS Loudness Radar & Stem Switcher**: Live Integrated LUFS, Short-Term LUFS, and True Peak (`dBTP`) metering with `-14 LUFS (YouTube / Reels)` and `-16 LUFS (Apple Podcasts)` target guides, plus instant **Audio Stem Switcher** (`All Stems`, `Mic Only`, `Desktop Audio`, `System Sound`).
  - **Color Scopes, 400% Pixel Magnifier Loupe & Safe Guides**: Live **RGB Parade & Luminance Waveform** scopes, interactive **`400%` Pixel Magnifier Loupe** for UI text sharpness inspection, and toggleable **`16:9 YouTube`**, **`9:16 Reels`**, **`1:1 Square`**, and **`4:5 Social`** safe-zone overlays.
  - **Lossless Instant Cut (`I` / `O` keys) & 4K Frame Snapshot**: Sub-second lossless MP4 container remux export (`⚡ Lossless Cut (<0.5s)`) and **`📸 4K Frame PNG`** snapshot directly to `<recordings_dir>/Exports/` and the macOS Clipboard.
- **`"Play With..."` Context Modal & macOS Default Player Registration (`platform.rs` & `tauri.conf.json`)**:
  - Choosing to play a clip opens the **`"Play With..."`** modal with **`⚡ sricap Studio Pro Player`** (`PRO DEFAULT`), **`🎬 QuickTime Player`**, **`🍊 VLC Media Player`**, and **`✨ IINA / Movist Pro`**.
  - Includes persistent **`[✓] Always open with sricap Studio Player`** setting (`store.rs`) and 1-click **`🌐 Set sricap as macOS Default Video Player`** via macOS LaunchServices (`LSSetDefaultRoleHandlerForContentType` for `.mp4`, `.mov`, `.mkv`, `.webm`, `.m4v`).

### 3. Universal $200 Tier Book & Document Reader Studio (`reader_window.rs` & `main_window.rs`)
- **Dedicated Header Trigger (`open-book-reader`, `icons/book-open.svg`)**:
  - Opens [`ReaderWindow`](file:///Users/sr20536224wipro.com/Documents/Github-apple/srinew/Cap/apps/desktop-gpui/src/reader_window.rs) supporting **PDF, EPUB, MOBI, AZW3, DJVU, Markdown (`.md`), HTML, AsciiDoc, Org-Mode, and CBZ/CBR**.
  - **Typography & Bionic Reading Engine**: 4 reading themes (`Warm Paper`, `E-Ink Mono`, `Dark Obsidian`, `Solarized Sepia`), 3 font families (`Serif (Charter)`, `Sans (Inter)`, `Mono (JetBrains)`), **Bionic Reading Mode (`+40% Speed` guided fixation bolding)**, Two-Page Kindle Spread toggle, Zen Fullscreen mode, and **Apple Neural TTS (`0.5x`–`3.0x`)**.
  - **Smart Learning, Highlights & AI Study Companion**: Color-coded highlights (`🟡 Core Concept`, `🟣 Key Quote`, `🔴 Counterargument`), sticky notes, interactive TOC & Bookmarks sidebar, Inline Terminology Inspector, 1-click **`✨ Synthesize Chapter Summary`**, and 1-click **`📋 Export Markdown Notes & AI Flashcards`** (`BOOK_LEARNING_PROTOCOL.md`).

### 4. 20 Flagship Broadcast, Network Radar & Player Novelty Features (Completed Checklist)
- [x] **#1 Zero-Cost Global Free-to-Air TV Directory**: Native `iptv-org` verified legal public channels (NASA TV 4K, Bloomberg TV, DW English, France 24, NHK World, PBS America, Euronews, WeatherNation) + `.m3u8` playlist loader.
- [x] **#2 Apple TV Style Live EPG**: Visual Electronic Program Guide showing current & upcoming shows with live progress percentage.
- [x] **#3 Multi-View Sports & News Mosaic (4-Channel 2×2 Grid)**: Simultaneous 2×2 multi-stream grid with 1-click active audio switching.
- [x] **#4 DVR Catch-Up & 30-Minute Live Pause Buffer**: Rolling 30-minute live DVR rewind (`-30s` steps up to `-1800s`) and pause buffer.
- [x] **#5 Smart Auto-Fallback Stream Resolver**: Automatic mirror CDN failover (`Primary CDN` ↔ `Mirror CDN`) without interrupting playback.
- [x] **#6 Hardware-Decoded Low-Latency HLS/DASH Engine**: Sub-second (`0.28s`) VideoToolbox zero-copy channel tuning.
- [x] **#7 Picture-in-Picture (PiP) Floating Glass HUD**: Detachable floating PiP mode for watching live TV or tutorials while coding.
- [x] **#8 Real-Time Live Closed-Captions & Subtitle Translator**: On-the-fly Whisper live closed captions & instant translation (`EN`, `ES`, `DE`, `FR`, `JA`, `HI`).
- [x] **#9 Instant Stream Clip to Studio Editor (`Cmd+Shift+R`)**: 1-click capture of the last 30 seconds of any live stream directly into `<recordings_dir>/Exports/` and the Cap Studio Editor.
- [x] **#10 Audio-Only Background Radio Mode**: `~64 kbps` low-bandwidth background audio mode for global news and tech streams.
- [x] **#11 Wi-Fi 6E/7 BSSID Radar & Spectrum Analyzer**: Displays BSSID, channel load percentage, and 5/6 GHz co-channel interference telemetry.
- [x] **#12 Streaming Bufferbloat Grade & QoS Optimizer**: Evaluates loaded latency delta (`A+` through `F` grade) with 1-click DSCP QoS buffer pacing.
- [x] **#13 CDN Edge Latency Map**: Probes Cloudflare, Fastly, AWS CloudFront, and Akamai POPs to recommend the optimal upload region.
- [x] **#14 Packet Loss Forensics & Jitter Waveform**: Live rolling 16-bar oscilloscope waveform of network jitter inside the Telemetry Modal.
- [x] **#15 Auto-Throttle Bandwidth Saver**: Toggleable metered-connection / battery-saver bandwidth limiter.
- [x] **#16 AI Speech Clarity & Dialog Boost**: Multi-band vocal enhancer (`+4.5 dB` presence boost) isolating speech over background music.
- [x] **#17 Dynamic Volume Night Mode**: Soft-knee transient limiter (`-16 LUFS` / `-2.0 dBTP` ceiling) smoothing loud spikes.
- [x] **#18 Precision Subtitle Resync Waveform**: Millisecond subtitle offset controls (`Sub -50ms` / `Sub +50ms`) for exact lip sync.
- [x] **#19 Lossless Frame Extraction & Color Sampler**: 10-bit 4K PNG frame snapshot + live Hex/RGB Rec.709 color sampler readout.
- [x] **#20 macOS Spatial Audio & Head-Tracking Support**: Multi-channel soundstage switcher (`Reference Stereo`, `Spatial Binaural`, `Spatial 7.1.4 Head-Tracked`).

### 5. Advanced Transcription Model Hub & Disk Cleaner (`editor_tabs.rs`, `settings_pages.rs`, & `transcription.rs`)
- [x] **Model Cards with Exact Sizes & Roles**: Lists `NVIDIA Parakeet INT8 (670 MB)`, `Parakeet Max (2.55 GB)`, `Whisper Small (466 MB)`, and `Whisper Medium (1.53 GB)` with live download status, **`Primary Engine`** and **`Secondary / Fallback Engine`** role toggles, 1-click **`Download / Install`** with progress bar, and **`Delete Permanently`** disk cleanup.

### 6. Reverse-Engineered Parakeet-AI Call & Mock Interview Suite (`copilot_window.rs` & `main_window.rs`)
- [x] **Undetectable Screen-Share Stealth Overlay HUD (`open-copilot-window`)**: Uses macOS `NSWindowSharingNone` (`set_window_capture_hidden`) so Zoom, Google Meet, Teams, and Webex screen-sharing never capture the copilot window.
- [x] **Dual-Channel Speaker + Mic Audio Splitter & Pluggable `.env` LLMs**: Separates Interviewer (Speaker) and Candidate (Mic) streams for `<200ms` local NVIDIA Parakeet TDT transcription with pluggable **Groq LLaMA-3.3 70B (`GROQ_API_KEY`, ~300 tok/s)**, **Google Gemini 2.0 Flash (`GEMINI_API_KEY`)**, and **100% Offline Ollama (`OLLAMA_HOST`)**.
- [x] **Mock Interview Simulator, FAANG Question Bank, ATS Resume Matcher & Studio Headshots**: Includes interactive Neural TTS mock interviewer, post-call STAR scorecard (Technical Accuracy, STAR Completeness, WPM Pacing, Filler Word `%`), curated Apple/Google/Meta/Stripe/Amazon/Netflix question bank, ATS Resume Keyword Match Score (`0–100%`), and 1-click AI Studio Headshot generator.

---

## [v0.6.7] — 2026-10-23

### Argon Apple HIG Liquid Glass UI Redesign & Interactive Walkthroughs
- **Apple HIG Liquid Glass Aesthetic**: Multi-layered frosted obsidian surfaces, `1px` specular inner borders, strict 8-point spatial rhythm, and 2-row compact Studio Editor header bar.
- **Interactive Walkthrough Guides**: Added 3 self-contained HTML5 interactive walkthrough demos (`docs/interactive-demos/`) accessible directly from the Main Window header bar (`walkthrough-guides`).
- **Prebuilt Sidecar Release Caching**: `.github/workflows/release-sricap-mac.yml` caches and publishes `cap-sidecars-macos-arm64.tar.gz` (`cap-muxer` & `cap-cli`) so CI skips redundant sidecar compilation.

---

## [v0.6.6] — 2026-10-23

### 8K Ultra-Clarity Low-Bitrate Recording & Export Engine
- **Content-Adaptive Screen Recording (5–8× Smaller Files, Zero Blur)**:
  - **10-Bit HEVC (`hevc_videotoolbox` Main10)** & **H.264 High444 / High Profile** with **Adaptive Idle-Scene VFR** (drops frame emission during static IDE/terminal pauses while jumping back to 60/120fps on cursor/scroll motion).
  - **Smart Static/Motion ROI Bitrate Allocator**: Preserves `CRF 14` razor-sharp quality on terminal/code text while compressing webcam/video regions efficiently.

---

## [v0.6.5] — 2026-10-22

### Enterprise & Developer Innovation Suite (`crates/recording/src/telemetry.rs` & `editor_window.rs`)
- **30-Second Rolling DVR Bug Replay Buffer (`Cmd+Shift+S`)**: Captures the last 30 seconds of screen, audio, and semantic click/keystroke telemetry (`RollingDvrBuffer`) into an instant repro bundle.
- **AI Cinematic Auto-Director, Magnetic Cursor Snapping & Neural Time-Warp**: Automatic multi-angle cuts, physics-smoothed cursor snapping (`smooth_and_snap_cursor_trajectory`), and `8x` fast-forward over stagnant terminal builds.
- **Smart Keystroke Password Masking, Playwright/Maestro E2E Codegen & C2PA Provenance**: Automatically masks secrets typed into password fields, exports `.spec.ts` (Playwright) and `.yaml` (Maestro) E2E tests from recordings, and embeds C2PA cryptographic provenance manifests + forensic luma watermarks.

---

## [v0.6.4] — 2026-10-22

### Apple & Android Pro Mirror Hub (`android_window.rs` & `device_bridge.rs`)
- **Unified Mobile Mirroring Studio**: Low-latency Android (`adb` + Scrcpy 16-byte control socket) and Apple iPhone Mirroring / `simctl` iOS Simulator hub with Screen-Off Stealth Mode, Clean `9:41` Demo Status Bar, Telephony Call HUD, `Cmd+K` Dialer, Smart OTP 2FA Clipboard Bridge, Dual-Device Comparative Canvas, and Touch Ripple Visualizer (`touch_gestures.rs`).

---

## [v0.6.3-sricap] — 2026-10-21

### 1. Brand Identity, Packaging & Offline Pro Unlock
- **Standalone Rebrand (`sricap-rec`)**:
  - Renamed desktop binary, macOS `.app` bundle (`sricap-rec.app`), and installer disk image (`sricap-rec.dmg`) with bundle identifier `com.sri.sricap-rec`.
  - Generated custom Obsidian + Indigo/Violet Aperture Retina icons (`16×16` through `1024×1024` `.icns`, `dock-icon.png`, `logo-full.svg`, `logo-full-dark.svg`, and menu bar tray icons).
- **100% Offline Pro Mode Unlocked**:
  - Unlocked all Pro features locally (`AuthSnapshot::is_upgraded() -> true`) with zero cloud login or paywall gates: 4K/60fps recording & export, Studio Editor, AI Captions, Camera 3D/Blur, and Autopilot tools.
- **Automated macOS DMG Release Pipeline (`.github/workflows/release-sricap-mac.yml`)**:
  - Builds and caches both Rust workspaces (`1.88.0` sidecars + `1.95.0` GPUI) and native dependencies (`FFmpeg`, `ONNX Runtime`, `Spacedrive`).
  - Automatically builds and bundles `cap-muxer` and `cap-cli` sidecars and publishes `sricap-rec.dmg`, `sricap-rec-arm64.dmg`, `sricap-rec-arm64.zip`, and `SHA256SUMS.txt`.

---

### 2. Pre-Recording & Live Recording Controls
- **Google Meet / Teams-Style `[All Windows] | [Browser Tabs]` Picker**:
  - Clicking **Window** on the Main Recorder opens the Window & Browser Tabs picker directly.
  - **`All Windows (N)`**: Live 2-column thumbnail grid of all open desktop windows with search filtering.
  - **`Browser Tabs (M)`**: Enumerates open tabs across **Google Chrome, Arc, Brave, Microsoft Edge, and Safari** (with title, browser name, and host domain). Clicking any tab switches the browser to that tab, brings it to the front, and locks window capture onto it.
  - **`Or Hover on Screen to Select Window`**: 1-click fallback to interactive screen-hover window selection.
- **Pre-Recording Aspect Ratio Bar (`Area` Overlay)**:
  - Top glass bar with **`Full`**, **`16:9`**, **`9:16`**, **`1:1`** aspect-ratio locks and **`720p`**, **`1080p`**, **`1440p`**, **`4K`** resolution presets.
- **Floating Recording Controls Bar (`controls_window.rs`)**:
  - Added live **Microphone VU Meter** bar (`-48 dB` to `0 dB` green/amber/red peak indicator).
  - Added live **Camera Bubble Show/Hide** toggle button during active recording.
  - Added live **Teleprompter Show/Hide** toggle button during active recording.
- **Floating Camera Bubble Quick-Controls (`camera_window.rs`)**:
  - Hovering the live Camera Bubble reveals 4 size pills (**`S`**, **`M`**, **`L`**, **`XL`**) and 3 shape pills (**`Circle`**, **`Rect`**, **`Square`**).

---

### 3. Tech-Influencer 9:16 Vertical Studio (YouTube Shorts, Instagram Reels & TikTok)
- **3-Zone 9:16 Vertical Layout (`1080 × 1920` FHD / `2160 × 3840` 4K)**:
  - **Top Zone (`y = 17%`)**: Relocates the Camera Bubble to the upper third with Large Squircle or Circular framing, `3.5px` white border ring, and `72%` ambient shadow.
  - **Center Stage (`y = 53%`)**: Centers the active Terminal or Browser window wrapped in dark macOS Window Chrome (red/yellow/green traffic lights) or Browser URL Bar frame.
  - **Bottom Safe Zone (`y = 80%`)**: Positions word-by-word animated captions above the bottom `250px+` protected zone so Instagram/YouTube UI buttons never cover code or text.
- **1-Click Creator Presets (`Background → Layout`)**:
  - **`16:9 Desktop`** | **`9:16 Shorts & Reels`** | **`1:1 Post`**
  - **`Terminal Dev Reel (9:16)`**: Deep Carbon gradient, dark macOS terminal window chrome (`zsh — terminal`), Top-Center Squircle Camera, `1.35x` cursor-follow auto-zoom, modifier keystroke badges enabled, `+0.20` text contrast boost, and `#00FF66` Dev Green 3-word captions.
  - **`Web Demo Reel (9:16)`**: Midnight Purple mesh gradient, dark Browser window chrome (`app.cap.so`), Top-Center Circular Camera, `1.55x` click-driven auto-zoom, `+0.15` contrast boost, and `#FFE600` Hormozi Yellow 3-word captions.
- **Creative Brand Overlays, Multi-Image Sequence Manager & Hook Banners (`Background → Layout`)**:
  - **`Upload Images (2s Default)`**: Multi-select image picker (`NSOpenPanel` / `rfd`) that imports multiple PNG/JPG/WebP/GIF images at once, numbers them `#1`, `#2`, `#3`..., and sequences them back-to-back at **`2.0s` each** starting from the playhead.
  - **Numbered Image & Slide Sequence Manager (`#1`, `#2`...)**:
    - Click any image header (`#1 Slide.png (0.0s–2.0s · 2.0s)`) to focus it on the timeline and select its **Canvas Drag & Resize Handles** so you can drag it anywhere on the video frame.
    - **Reorder (`↑` / `↓`) & Auto-Sequence**: Swap image order (`↑` / `↓`) while automatically swapping timeline slots, or click **`Sequence All (2s Each)`** / **`Sequence All (3s Each)`**.
    - **Per-Image Duration Presets**: Set any image's visibility duration in 1 click (**`1s`**, **`2s` (Default)**, **`3s`**, **`5s`**, **`Full`**) or drag its start/end handles on the **Image** timeline track.
    - **Position Presets**: **`Center`**, **`TL`**, **`TR`**, **`BL`**, **`BR`** + **`Del`** to remove any image.
  - **`Add Reel Hook Banner`**: Inserts a bold top-anchored headline pill (`"HOW TO BUILD THIS IN 30s"`, `#4F46E5` pill backdrop, `800` weight, `2px` outline) on the Text track.
- **40-Tool Creator Studio Suite (`Background → Layout`, `Audio`, & `Export`)**:
  1. **`+ Intro (2s)` Card Generator**: Branded 2-second Intro Title Card at `0:00`.
  2. **`+ Outro CTA` Card Generator**: 2.5-second `"FOLLOW FOR PART 2 · LIKE & SUBSCRIBE"` End Screen.
  3. **`+ Lower-Third` Speaker Badge**: Animated speaker name & handle pill (`0.4s – 3.9s`).
  4. **`+ Step ①②③` Auto-Incrementing Callouts**: Numbered `① STEP 1`, `② STEP 2`, `③ STEP 3` callout pills at the playhead (`2.5s` duration, draggable on canvas).
  5. **`+ Spotlight` Focus Mask**: `MaskKind::Highlight` (`68%` dim surround, `14%` soft feather) at the playhead (`3.0s` duration, draggable/resizable on canvas).
  6. **Multi-Clip Transitions (`Cut (No Trans)` | `Crossfade Clips` | `Fade-Through`)**: Smooth `0.35s` transitions across split/merged video clips.
  7. **Cinema Color Grade LUT Presets (`Crisp Dev` | `Warm Face` | `Cyberpunk` | `Keynote`)**: 1-click color grading for Screen and Camera.
  8. **`Cursor Boost`**: Toggles `195%` high-visibility mobile cursor sizing with smooth spring physics.
  9. **`Cover .PNG` / `Save Cover Thumbnail (.png)`**: Renders the current composited GPU frame to `<recordings_dir>/Exports/<title>-thumbnail.png` and copies it to the macOS Clipboard.
  10. **`Export Split Clips as Separate Chapter MP4s (01, 02…)`**: Batch-exports each split segment as `<title>-01.mp4`, `<title>-02.mp4`... in `Exports/`.
  11. **`Talking Head` Scene Switcher**: Inserts a `2.5s` full-screen webcam `SceneMode::CameraOnly` segment at the playhead with smooth transitions.
  12. **`Hide Cam 3s` Scene**: Temporarily hides the camera bubble for `3.0s` at the playhead (`SceneMode::HideCamera`).
  13. **`+ 3D Tilt` Hero Keyframe**: Inserts a `3.0s` Apple-style 3D perspective screen tilt (`rotate_y = -14°`, `rotate_x = 8°`, `zoom = 1.10x`) at the playhead.
  14. **`Freeze 2s` Hold Frame**: Splits a micro-slice at the playhead and slows it to `0.04x` so the video freezes for ~2 seconds while voiceover/callouts continue.
  15. **`+ Progress` Retention Bar**: Generates 4 sequential progress bar indicators (`25%` → `50%` → `75%` → `100%`) across the video duration.
  16. **`+ CLI Box` Terminal Command Overlay**: Inserts a dark terminal command snippet card (`#090D16` bg, `#00FF66` neon green text) at the playhead.
  17. **`+ Pro-Tip` Callout Card**: Inserts an amber/gold `"⚡ PRO TIP"` badge at the playhead.
  18. **`Zoom Steps` Auto-Zoom Sync**: Automatically creates `1.55x` manual-target `ZoomSegment`s aligned to every Step Callout, Pro-Tip, and Spotlight mask on the timeline.
  19. **`Podcast Voice Master EQ` (`Audio` Tab)**: Enables Studio Sound voice isolation, `+2.5 dB` mic boost, and `-14 dB` background/system audio ducking in 1 click.
  20. **`Export Subtitles & Docs Bundle (.SRT + .VTT + .TXT + .MD)` (`Export` Drawer)**: Writes `<title>.srt`, `<title>.vtt`, `<title>-transcript.txt`, and `<title>-sop.md` to `<recordings_dir>/Exports/`.
  21. **`+ Quiz Card` ("Pause & Guess Output 🤔") (`Background → Layout`)**: Inserts a 3.0s interactive challenge card (`#312E81` indigo bg, `#FACC15` gold text) at the playhead.
  22. **`+ Key ⌘K` Custom Shortcut Pill (`Background → Layout`)**: Inserts a custom keyboard shortcut pill (`"⌘ + SHIFT + P"`, `#18181B` keycap bg, `#6366F1` border) at the playhead.
  23. **`Before/After` Comparison Labels (`Background → Layout`)**: Inserts synchronized `"✕ BEFORE (SLOW)"` (red) and `"✓ AFTER (10X FASTER)"` (green) comparison cards at the playhead.
  24. **`+ Arrow ➔` Pointer Callout (`Background → Layout`)**: Drops a directional `"➔ WATCH THIS LINE"` red pointer badge at the playhead, selected on canvas for dragging.
  25. **`60s Reel Cut` Auto-Teaser Condenser (`Background → Layout`)**: Cuts filler words/silences and condenses the timeline into a `<= 58s` YouTube Short / Instagram Reel.
  26. **`Jump-Cut Zoom` Alternating 1.28x Zoom (`Background → Layout`)**: Automatically applies `1.28x` zoom on alternating clip segments so jump cuts look like a multi-camera edit.
  27. **`Pace 1.15x All` Creator Pacing (`Background → Layout`)**: Speeds up all timeline clips to `1.15x` for brisk tutorial pacing.
  28. **`+ Lo-Fi Ambient Pad` Royalty-Free Music Synthesizer (`Audio` Tab)**: Generates a warm 48kHz Lo-Fi chord progression pad `.wav` (`-14 dB`) natively in Rust across the video duration.
  29. **`+ Censor Bleep (1s)` (`Audio` Tab)**: Synthesizes and inserts a `1000 Hz` censor bleep `.wav` at the current playhead.
  30. **`Export All 3 Formats (16:9 YouTube + 9:16 Reels + 1:1 Square)` (`Export` Drawer)**: Batch-renders all 3 aspect ratios (`16:9`, `9:16`, and `1:1`) into `<recordings_dir>/Exports/` in 1 click.
  31. **`YT 4K DevOps (16:9)` & `Export YouTube 4K DevOps/Coding (Max Screen · Compact Size)`**:
      - Maximizes screen area (`padding = 2.0%`, `rounding = 10.0`) so 98% of the `3840×2160` 4K canvas is your Terminal / IDE / Cloud Console, with a compact `20%` bottom-right webcam, `+0.22` code contrast boost, `1.4x` smart click auto-zoom, and `0.08 bpp` H.264 High Profile encoding for **70% smaller 4K file sizes**.
  32. **`Save Brand` & `Apply Brand` Default Brand Kit (`Background → Layout`)**: Saves and restores your exact layout, gradient, camera shape/border, cursor, and color grading (`sricap-brand-kit.json`) in 1 click.
  33. **`Auto-Hook 🔥` Viral Headline Generator (`Background → Layout`)**: Extracts the opening phrase from your transcript and generates a bold top Hook Banner automatically.
  34. **`Chapter Cards` Auto-Section Dividers (`Background → Layout`)**: Inserts `1.8s` `PART 1`, `PART 2`, `PART 3` chapter title cards at every split/merged clip boundary.
  35. **`Silences 3x` Time-Lapse Coding Pauses (`Background → Layout`)**: Splits silent intervals (`>0.9s`) and speeds them up to `3.0x` while keeping spoken segments at `1.0x`.
  36. **`Portrait Cam` DSLR Webcam Glow (`Background → Layout`)**: Applies warm skin-tone color grading, contrast boost, `4px` indigo glow ring, and `82%` ambient shadow to the webcam.
  37. **`Safe Guide` 9:16 Social Safe-Zone Overlay (`Background → Layout`)**: Toggles non-destructive Reels/Shorts safe-zone guide boundaries on the video canvas.
  38. **`Loop Reel ↺` Seamless Ending-to-Intro Bridge (`Background → Layout`)**: Adds crossfade transitions and a `"↺ REWATCH FROM STEP 1"` bridge card at the end of the Reel.
  39. **`+ SFX Whoosh` & `+ SFX Ding Chime` (`Audio` Tab) + `Spread Across Video` (`Image Sequence Manager`)**: Synthesizes transition whoosh and notification chime cues at the playhead, and evenly spreads uploaded slide images across the timeline.
  40. **`Export CLI & Shortcut Cheat-Sheet (.MD)` (`Export` Drawer)**: Extracts all recorded shortcuts and callout cards into `<title>-cheatsheet.md` in `Exports/` and copies it to the macOS Clipboard.
  41. **Top Header `Save`, `Discard Edits`, `Discard & Close`, and `Quit App` Controls**:
      - **`Save`**: Writes project changes to disk immediately and updates the restore checkpoint.
      - **`Discard Edits`**: Reverts all unsaved experimental edits back to the last saved checkpoint without closing the window.
      - **`Discard & Close`**: Ignores unsaved edits, restores the last saved state on disk, closes the Studio Editor, and returns to the Main Window.
      - **`Quit App`**: Cleanly exits `sricap-rec` in 1 click.
  42. **Top Header 1-Click Platform Aspect Ratio Switcher (`16:9 YouTube` | `9:16 Reels` | `1:1 LinkedIn`)**:
      - **`16:9 YouTube`**: Applies the 4K DevOps/Coding Max-Screen preset (`padding = 2.0%`, `rounding = 10.0`, bottom-right 20% webcam, +0.22 code contrast).
      - **`9:16 Reels`**: Applies the 3-zone 1080×1920 vertical teaching layout for Instagram Reels, YouTube Shorts, and TikTok.
      - **`1:1 LinkedIn`**: Applies the 1:1 Square feed layout (`AspectRatio::Square`, tight `4%` padding, bottom-right square webcam, centered captions).
  43. **`Trim Start [` & `Trim End ]` (`Background → Layout`)**: 1-click ripple-trim of dead air before or after the current playhead on the active clip.
  44. **`Duplicate` Selected Segment (`Background → Layout`)**: Duplicates the selected `Text`, `Image`, `Mask`, or `Zoom` segment and places the copy right after it.
  45. **`+ Pixelate` Sensitive Area Mask (`Background → Layout`)**: Inserts a `MaskKind::Sensitive` pixelation box (`24px` blocks, `3.0s` duration) at the playhead and activates its canvas drag/resize handles.
  46. **`Split Scene` & `Floating Dual` (`Background → Layout`)**: Inserts a `SceneMode::SplitScreen` or `SceneMode::Floating` dual-card layout (`3.0s` duration) at the playhead.
  47. **`Clear Overlays` (`Background → Layout`)**: Clears all added Text, Mask, Scene, and 3D overlays in 1 click while preserving video clips.
  48. **`Exports Dir` (`Background → Layout`)**: Opens `<recordings_dir>/Exports/` directly in macOS Finder.
  49. **`+ Link Card 🔗` & `+ QR / Docs 📱` + `Copy Clickable SOP & Chapters (.MD)`**: Inserts floating process link / QR callout cards at the playhead and exports clickable Markdown chapters (`[MM:SS](#t=Xs)`).
  50. **Viral & Coding Caption Typography & Presets (`Hormozi Fire`, `Terminal CLI`, `MrBeast Gold`, `Glass Pill`, `Editorial Serif`, `Dev Green`, `JetBrains Mono / SF Mono`, `Impact / Heavy Display`, `Inter / Crisp UI`) + `Auto All-Caps & Punctuation Optimizer` + Strict Single-Window `SCContentFilter` Isolation**:
      - True macOS `SCContentFilter::with_desktop_independent_window` single-window & browser-tab isolation so overlapping terminals or notifications never leak into window recordings.
  51. **Opt-In Cursor Auto-Zoom Toggle (`Auto-Zoom: OFF` / `Auto-Zoom: ON`)**:
      - Cursor Auto-Zoom is **OFF by default** across all presets (`16:9 YouTube`, `9:16 Reels`, `DevOps Reel`, `Web Reel`, `Magic Polish`) so screen recordings stay at a steady 100% view unless explicitly enabled via the 1-click **`Auto-Zoom: OFF / ON`** toggle in the Studio Editor Header Bar or `Background → Layout` sidebar.
  52. **Direct `Open Studio Editor` & `Import .MP4 / Video` on Main Page + In-Editor `+ Add .MP4`, `+ Attach Cam (.mp4)`, & `+ Record Cam/Clip`**:
      - Prominent **`Open Studio Editor`** and **`Import .MP4 / Video`** buttons on the Main Page (`main_window.rs` & `import.rs`) that open the Studio Editor directly (auto-creating a blank `Studio Canvas` project if none exists).
      - Top Header Bar **`+ Add .MP4`** and **`+ Record Cam/Clip`** buttons inside Studio Editor.
      - Always-enabled **`Camera`** sidebar tab with **`+ Attach Cam (.mp4)`** (`attach_camera_video_clicked`) to attach any external `.mp4`/`.mov` webcam video onto an imported MP4 or screen recording, plus **`+ Record Cam/Clip`**.
  53. **6-Stage Audacity-Grade Audio DSP & 1-Click Studio Voice Profiles + `iPhone Bezel` / `MacBook Bezel` Frames + Teleprompter `Slow`/`Med`/`Fast` Presets**:
      - `Podcast Studio`, `Tech Screencast`, and `Noisy Room Fix` 1-click profiles in the `Audio` tab (`crates/audio/src/voice_level.rs` + `editor_tabs.rs`).
      - `iPhone Bezel` (Dynamic Island + squircle bezel) and `MacBook Bezel` (`FrameStyle::MacOS` dark window chrome) buttons in `Background → Layout`.

---

### 4. Studio Autopilot, Transcript Editing, Camera & Audio Tools
- **1-Click `"Magic Polish (All-in-One)"` (`Captions` Tab)**:
  - Automatically cuts filler words (`um`, `uh`, `ah`, `er`, `hmm`) and dead-air silences (`> 1.0s`).
  - Generates `1.5x` click-driven `ZoomSegment`s with spring cursor tracking.
  - Generates shortcut-only `KeyboardSegment` badges (`⌘C`, `⌘V`, `⌘S`).
  - Enables Studio Sound voice isolation, `-12 dB` audio ducking, and studio background padding/rounding/shadow.
- **Transcript-Driven Editing & Documentation (`Captions` Tab)**:
  - **Social Caption Presets**: **`Pop` (`#FFE600` Hormozi Yellow)**, **`Dev Green` (`#00FF66`)**, **`Minimal`**, **`Highlight`**, **`Classic`** with automatic 3-word-per-line chunking (`regroup_captions_for_social_reels`).
  - **`Remove Filler Words (N)`**: Ripple-deletes detected filler words across the timeline.
  - **`Remove Silences (>1.0s) (N)`**: Ripple-deletes dead-air pauses.
  - **`Auto-Redact Sensitive Info (N)`**: Scans transcript and screen for emails, API keys, tokens, phones, and cards and inserts blur `MaskSegment`s.
  - **`Copy YouTube / Loom Chapters`**: Generates timestamped `00:00 Intro` chapters onto the clipboard.
  - **`Export Step-by-Step SOP (.md)`**: Writes a Markdown SOP guide with timestamps and action items to `<project>/sop-guide.md` and copies it to the clipboard.
- **Enhanced Camera Tab (`Camera` Tab)**:
  - **Rapid Size & Split-Screen Presets**: **`Small 25%`**, **`Med 40%`**, **`Lg Square 60%`** (top-anchored rounded squircle), and **`Split 50/50`** (top-half Square camera + bottom-half screen display).
  - **Shape Presets**: **`Circular`**, **`Rounded Rect`**, **`Square`**.
  - **Camera Border**: Toggle with customizable **Border Width (`1–20px`)** and **Border Opacity (`0–100%`)**.
  - **3D Perspective Tilt**: **`Flat`**, **`Subtle 3D`**, **`Dynamic 3D`**, **`Isometric`**.
- **Enhanced Multi-Track Audio Tab (`Audio` Tab)**:
  - **`Import Background Audio (.mp3 / .wav)`**: Native file picker to add background music or voiceover tracks (`AudioTrackSegment`).
  - **`Background Music Volume`**: 1-click level presets (**`25% (-18dB)`**, **`50% (-12dB)`**, **`75% (-6dB)`**, **`100% (0dB)`**).
  - **`Auto-Ducking (-12 dB under voice)`**: Lowers both system audio and imported background music tracks by `-12 dB` when microphone speech is present.
  - **`Remove All Pauses (>1.0s)`** & **`Export Audio (.wav)`**.

---

### 5. Ultra-Sharp 4K/60fps & Social Export Engine
- **Upgraded Export Defaults (`editor_export.rs` & `crates/enc-ffmpeg/src/h264.rs`)**:
  - Defaults to **`4K`**, **`60 FPS`**, and **`Maximum` (`0.45 bpp`)** quality.
  - Upgraded `h264_videotoolbox` and `libx264` to **H.264 High Profile** (`8×8 DCT` + `CABAC`, `CRF 16`) for razor-sharp terminal and code text.
- **1-Click Export Presets (`Export` Drawer)**:
  - **`Export for Reels & Shorts (1080×1920 · 60fps)`**: Locks output to `1080 × 1920 @ 60 FPS`, H.264 High Profile, `yuv420p`, `0.12 bpp` (~14.2 Mbps VBR).
  - **`Copy Reels & Shorts MP4 to Clipboard`**: Renders the `1080 × 1920 @ 60 FPS` vertical MP4 and copies it directly to the macOS Clipboard for instant pasting into AirDrop, Slack, Telegram, or web uploaders.
  - **`Copy PR / Slack GIF (1080p · 20fps)`**: Renders a crisp `1080p @ 20 FPS` GIF and copies both file URL and raw `com.compuserve.gif` bytes to the macOS Clipboard.
- **macOS Clipboard & Auto-Save Reliability (`platform.rs`)**:
  - Canonicalizes paths (`std::fs::canonicalize`), writes `public.file-url`, `NSFilenamesPboardType`, and `com.compuserve.gif` to `NSPasteboard`, and saves all clipboard exports in `<recordings_dir>/Exports/`.

---

### 6. Local Storage Management & Fast Developer Workflow
- **Local Storage Controls (`Settings → General → Recordings` & `Recordings Library`)**:
  - Live **Storage Used (`MB / GB`)** badge across `N` recordings.
  - **`Folder`**, **`Change`** (custom recordings folder picker), and **`Reset`** buttons.
  - **`Delete All Local Recordings`** button to reclaim disk space in 1 click.
- **Fast Incremental Dev Launch (`./start.sh` & `./stop.sh`)**:
  - Caches GPUI patch hashes in `target/.cap-gpui-patch-stamp` to skip redundant `prepare-gpui-dependency.sh` runs.
  - Uses `split-debuginfo = "unpacked"` and `codegen-units = 256` on `[profile.dev.package.cap-desktop-gpui]` for 2–5 second incremental rebuilds.
- **In-App Auto-Updater via GitHub Releases (`apps/desktop-gpui/src/installer.rs` & `src/updates.rs`)**:
  - Queries `https://api.github.com/repos/srinivaskona7/Cap/releases/latest` (`parse_github_release`), displays the release changelog preview, downloads `sricap-rec-macos-arm64.zip` in the background, unpacks with `/usr/bin/ditto -x -k`, and hot-replaces `/Applications/sricap-rec.app`.
- **Fast `sccache` Parallel CI & Polished macOS DMG (`.github/workflows/release-sricap-mac.yml`)**:
  - Uses `mozilla-actions/sccache-action@v0.0.7` (`RUSTC_WRAPPER=sccache`), compiles `cap-muxer`/`cap-cli` sidecars and `sricap-rec` GPUI binary in parallel, and builds a polished `.dmg` with custom `.VolumeIcon.icns` and Finder-positioned `sricap-rec.app` (`{175, 190}`) + `/Applications` drag-and-drop shortcut (`{425, 190}`).
