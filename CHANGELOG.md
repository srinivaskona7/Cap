# Changelog — `sricap-rec` Custom Studio & Screen Recording Suite

All notable features, presets, UI options, and architectural customizations built into **`sricap-rec`** (`com.sri.sricap-rec`) on the `feature1` branch.

---

## [v0.6.1-sricap] — 2026-10-21

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
- **Creative Brand Overlays & Hook Banners (`Background → Layout`)**:
  - **`Add PNG Logo / Watermark`**: Imports a PNG/WebP/JPG logo onto the Image track with 1-click corner placement pills (**`Logo TL`**, **`Logo TR`**, **`Logo BL`**, **`Logo BR`**) stretched across the full video duration at `92%` opacity.
  - **`Add Reel Hook Banner`**: Inserts a bold top-anchored headline pill (`"HOW TO BUILD THIS IN 30s"`, `#4F46E5` pill backdrop, `800` weight, `2px` outline) on the Text track.

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
