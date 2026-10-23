# Executive UI/UX Design & Engineering Specification for Argon

**Target App**: `sricap-rec` (Cap GPUI Next-Gen Native macOS Desktop App)  
**Objective**: Elevate the entire UI/UX to Apple Human Interface Guidelines (HIG) perfection, modern glassmorphism, ergonomic start screen organization, and world-class creator studio editing aesthetics.

---

## 🎨 1. Aesthetic Design System & Visual Tokens

### A. Liquid Glass & Frosted Blur Palette
* **Window Surface**: Deep multi-layered translucent obsidian glass.
  - Light mode: `rgba(255, 255, 255, 0.72)` with `backdrop-filter: blur(28px) saturate(190%)`.
  - Dark mode: `rgba(18, 19, 24, 0.78)` with `backdrop-filter: blur(32px) saturate(210%)`.
* **Border Highlights**: 1px subtle inner rim stroke with specular highlight (`rgba(255, 255, 255, 0.12)` top, `rgba(0, 0, 0, 0.35)` bottom) to produce native macOS Sonoma/Sequoia depth.
* **Accent Hierarchy**:
  - Primary Electric Indigo: `#3B82F6` -> `#6366F1` gradient.
  - Recording State: Pulsing ember red `#EF4444` with soft 8px ambient glow.
  - Studio Gold/Pro: `#F59E0B` metallic badging.

### B. Typography & Micro-Hierarchy
* Primary UI Font: San Francisco (`SF Pro Display` for headers, `SF Pro Text` for controls).
* Monospace Font: `SF Mono` / `JetBrains Mono` for timecodes, bitrates, resolutions, and hotkeys.
* Micro-labeling: Crisp uppercase 9px/11px badges with `letter-spacing: 0.05em` for mode tags (`STUDIO`, `INSTANT`, `4K RETINA`).

---

## 🚀 2. Start Page / Main Recorder HUD Reorganization

### The "3-Zone" Ergonomic Layout
Replace stacked clutter with a clean, instantly scannable 3-tier hierarchy:

```
┌────────────────────────────────────────────────────────┐
│  🔴 🟡 🟢  sricap   [Pro ⚡]           ⚙️ 📁 🔔 ⎋ ✕    │ ← Header (Pillars & Nav)
├────────────────────────────────────────────────────────┤
│   [ ⚡ Instant ]   [ 🎬 Studio Pro ]   [ 📸 Snapshot ]  │ ← Pill Segment Controller
├────────────────────────────────────────────────────────┤
│  CAPTURE SOURCE                                        │
│  ┌───────────────────────┐  ┌───────────────────────┐  │
│  │ 🖥️ Full Screen (Retina)│  │ 🪟 Active Window (App)│  │ ← Split Quick Tiles
│  │ 1728x1117 • 120Hz     │  │ Chrome, VS Code, etc. │  │
│  └───────────────────────┘  └───────────────────────┘  │
│  ┌───────────────────────┐  ┌───────────────────────┐  │
│  │ 📐 Custom Area Crop   │  │ 📱 Android / Mobile   │  │
│  └───────────────────────┘  └───────────────────────┘  │
├────────────────────────────────────────────────────────┤
│  AV INPUTS (Live Meters)                               │
│  📷 FaceTime HD Camera  [On 🟢]  ⚙️ Formats            │
│  🎙️ MacBook Pro Mic     [On 🟢]  [ ▂▃▅▆▇ Live VU ]    │ ← Real-time responsive VU
│  🔊 System Audio        [On 🟢]                         │
├────────────────────────────────────────────────────────┤
│  [  🔴 START 4K RECORDING  ( Space / ⌥R )  ]           │ ← High-Affordance CTA
└────────────────────────────────────────────────────────┘
```

### Key Start Screen Upgrades:
1. **Interactive Audio VU Meter**: Rather than a static toggle, give the microphone row a responsive, dancing green-to-amber LED level meter so creators know their audio is working *before* hitting record.
2. **One-Click Target Switching**: Add clear visual badge chips for **Full Screen**, **Window**, **Area**, and **Android Mirror** with instant thumbnail previews.
3. **App Quick-Action Buttons**: Add persistent **Settings (⚙️)**, **Recordings Library (📁)**, and a prominent **Close/Quit (✕)** button in the top title bar.

---

## 🎬 3. Studio Timeline & Editor Suite Improvements

### A. Magnetic Multi-Track Timeline
1. **Visual Track Hierarchy**:
   - Track 1: Screen Recording with mini live scrub thumbnails.
   - Track 2: Camera PIP bubble with shape modifiers (Circle, Squircle, Rounded Rectangle).
   - Track 3: Microphone Speech waveform with color-coded word transcription boundaries.
   - Track 4: Background Music / SFX track.
2. **Snapping & Ripple Deletes**:
   - Splitting a clip automatically provides magnetic snapping to playhead and silence markers.
   - Ripple delete automatically collapses empty space when an unwanted take is deleted.

### B. Inspector Panel (Right Sidebar)
Organize controls into clean collapsible accordion cards:
* **Camera PIP Inspector**:
  - Size slider (Small, Medium, Large, Custom %).
  - Shape picker & border shadow intensity.
  - Eye Contact / Neural Gaze Correction toggle.
* **Canvas Stage & Background**:
  - Wallpaper presets (Sunset Mesh, macOS Sonoma Sonoma Dark, Radial Glow, Studio Gray).
  - Corner radius (0px to 32px) and 3D Tilt isometric angle sliders.
  - Drop shadow spread and ambient bloom.
* **Audio Mastering & Polish**:
  - **Studio Voice Shield** (`nnnoiseless` toggle to strip all AC/fan noise).
  - Keystroke & Mouse Click Audio Enhancer.
  - Smart Ducking level slider (-12dB default).

### C. Export Modal ("One-Click Perfection")
* Quick Preset Chips:
  - **YouTube 4K / 1440p (Crystal Crisp, 60 FPS)**
  - **Slack & Discord (< 25 MB Ultra-Compressed 1080p)**
  - **Twitter / X & LinkedIn (16:9 or 9:16 Vertical Reel)**
  - **GIF / Animated WebP with loop controls**
* Estimated file size badge that updates dynamically as the user drags the bitrate/resolution slider!

---

## 📋 4. Exact Prompt You Can Copy-Paste Directly to Argon

```markdown
Hello Argon,

We are revamping the complete UI/UX and Start Page of our native macOS screen recording and editor app (Cap / sricap-rec) to make it look ultra-premium, modern, and aligned with Apple HIG standards.

Please implement the following design and layout specifications:

1. Visual Aesthetic & Liquid Glass Styling:
   - Apply macOS Sonoma/Sequoia frosted liquid glass (dark mode `rgba(18, 19, 24, 0.78)` with `backdrop-filter: blur(32px)` and a subtle 1px specular highlight border).
   - Use San Francisco font hierarchy (`SF Pro Display` for headers, `SF Pro Text` for controls, and `SF Mono` for timecodes/resolutions).
   - Use Electric Indigo (`#3B82F6` to `#6366F1`) as primary accent, and an ember-pulsing red with glow for active recording.

2. Start Page Reorganization (3-Zone Ergonomics):
   - Header Bar: Window drag handle, Logo with Pro badge, Quick navigation icons (Settings ⚙️, Recordings Library 📁, Notification 🔔, and Exit/Quit ✕).
   - Mode Selector: Segmented pill for [ ⚡ Instant Mode ], [ 🎬 Studio Pro ], and [ 📸 Screenshot ].
   - Capture Source Grid: 2x2 high-affordance rounded tiles for "Full Screen Display", "Active Window", "Area Selection", and "Mobile / Android Mirror" with live thumbnails.
   - Live Audio/Device Inputs: Include a real-time reactive dancing VU audio level meter on the microphone row so users can verify audio levels before recording.
   - Big Prominent CTA: Floating primary action button "START 4K RECORDING" with hotkey badge (`⌥R`).

3. Editor & Timeline Polish:
   - Magnetic multi-track timeline layout (Screen, PIP Camera, Voice Waveform, Music/SFX).
   - Right Inspector Panel organized into clean accordion cards: Canvas Background & 3D Tilt, Camera Shape & Gaze, and Studio Audio Denoise.
   - Export Sheet: Quick-preset chips for "Slack/Discord (< 25MB)", "YouTube 4K Crisp", and "Social Reel 9:16" with live file-size estimation.

Ensure all buttons have smooth micro-interactions (hover brightening, subtle press scale 0.98), rounded corners match macOS native curvature (10px–12px), and all text is high-contrast and legible.
```
