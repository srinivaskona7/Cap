# Cap Repository Memory Index

This directory contains structured, long-lived architectural and operational memory for the **Cap** (`CapSoftware/Cap`) monorepo, designed to work seamlessly across **OpenAI Codex**, **Claude Code**, and **Google Gemini CLI** with zero token waste and zero redundant discovery turns.

## Quick Orientation

- **Product**: **`sricap-rec`** (custom rebrand of Cap on branch `feature1`; `main` stays clean for upstream syncs) — open-source screen recording, 9:16 Reels/Shorts vertical teaching studio, screenshot annotation & async video messaging platform.
- **Monorepo Stack**: Turborepo + Bun (`1.4.0`) + Node (`>=20`) + Rust (`1.88.0` root workspace + `1.95.0` GPUI workspace).
- **Primary Shipping Desktop App**: `apps/desktop-gpui` (binary & bundle name: **`sricap-rec`**, bundle identifier: `com.sri.sricap-rec`, pure Rust GPUI app in a standalone Cargo workspace).
- **Legacy Desktop / Dev Harness**: `apps/desktop` (Tauri v2 + SolidStart + `specta` IPC; `src-tauri/binaries/` holds sidecar binaries `cap-muxer-<target>` and `cap-cli-<target>`).
- **Web Platform**: `apps/web` (Next.js 16 App Router, React 19, Effect-TS, Drizzle ORM + MySQL/PlanetScale, NextAuth + WorkOS SSO, S3/CloudFront, Tinybird analytics, MCP Server & Developer Platform).
- **Media Engine**: `crates/*` (50+ Rust crates for screen capture, camera, audio, `wgpu` shader rendering, hardware encoding, out-of-process MP4 muxing, and deterministic A/V verification).

## `sricap-rec` Custom Features & Direct File Map (`feature1` Branch)

| Feature / Subsystem | Exact File Paths & Symbols |
|---|---|
| **App Rebranding & Icons (`sricap-rec`)** | `apps/desktop-gpui/Cargo.toml` (`[[bin]] name = "sricap-rec"`), `apps/desktop-gpui/tauri.conf.json`, `apps/desktop/src-tauri/tauri.conf.json`, `apps/desktop/src-tauri/tauri.prod.conf.json` (`productName: "sricap-rec"`, `mainBinaryName: "sricap-rec"`, `identifier: "com.sri.sricap-rec"`), `apps/desktop-gpui/src/menus.rs` (`APP_NAME = "sricap-rec"`), `apps/desktop-gpui/assets/icons/logo-full*.svg`, `apps/desktop-gpui/assets/dock-icon.png`, `apps/desktop/src-tauri/icons/macos/icon.icns` |
| **Offline Pro Unlock, Direct Studio Launch & Local Storage Controls** | `apps/desktop-gpui/src/store.rs` (`AuthSnapshot::is_upgraded() -> true`), `apps/desktop-gpui/src/main_window.rs` (`Open Studio Editor` & `Import .MP4 / Video` buttons on Main Page + header `open-studio-direct`), `apps/desktop-gpui/src/import.rs` (`open_studio_editor_directly`, `create_new_blank_studio_project_and_open`, `synthesize_blank_studio_project`, `transcode_camera_overlay_for_project`), `apps/desktop-gpui/src/settings_window.rs` (`has_cap_pro = true`, custom recordings folder picker, `Open Studio Editor`, `Open Exports Folder`, storage used banner, `Delete All Local Recordings`), `apps/desktop-gpui/src/library.rs` (`compute_directory_size_bytes`, `delete_all_recordings`) |
| **Google Meet-Style Browser Tabs & Windows Picker + Strict Single-Window Isolation** | `apps/desktop-gpui/src/devices.rs` (`BrowserTabOption`, `list_browser_tabs_for_windows`, `activate_browser_tab` via AppleScript across Chrome, Arc, Brave, Edge, Safari + fallback window IDs), `apps/desktop-gpui/src/main_window.rs` (`browser_tabs_mode`, `[All Windows] | [Browser Tabs]` toggle in `render_target_grid`), `crates/scap-targets/src/platform/macos.rs` (`WindowImpl::as_content_filter` via `sc::ContentFilter::with_desktop_independent_window`, `WindowImpl::bring_to_front`), `crates/recording/src/sources/screen_capture/macos.rs` (`with_macos_window_id` isolated single-window capture) |
| **9:16 Vertical Teaching Studio, YouTube 4K DevOps Preset, Bezels & Multi-Image Sequence Manager** | `apps/desktop-gpui/src/editor_sidebar.rs` (`render_social_layout_presets`, `apply_tech_reel_preset`: 3-zone 1080×1920 layout with Top Camera at `y=0.17`, Center Framed MacOS/Browser Window at `y=0.53`, Bottom Safe Zone Hormozi/Dev-Neon Captions at `y=0.80`; `YT 4K DevOps (16:9)` `apply_youtube_4k_devops_preset` max-screen `2%` padding + `0.08 bpp` compact 4K; `Terminal Dev Reel (9:16)` & `Web Demo Reel (9:16)` presets; `1:1 LinkedIn` `apply_linkedin_square_preset`; `iPhone Bezel` & `MacBook Bezel` `apply_device_bezel_frame`; opt-in `Auto-Zoom: OFF / ON` `toggle_cursor_auto_zoom`; `Upload Images (2s Default)` multi-file picker, numbered `#1, #2...` sequence cards with `↑/↓` reorder, `1s/2s/3s/5s/Full` duration buttons, `Spread Across Video`, `Center/TL/TR/BL/BR` placement, and canvas drag focus; `Add Reel Hook Banner`) |
| **40-Tool Creator Studio Suite, Opt-In Auto-Zoom & Header Controls** | `apps/desktop-gpui/src/editor_window.rs`, `src/editor_sidebar.rs`, `src/editor_tabs.rs`, `src/editor_export.rs` (Header: `Save`, `Discard Edits`, `Discard & Close`, `Quit App`, `16:9 YouTube` / `9:16 Reels` / `1:1 LinkedIn` switcher, `Auto-Zoom: OFF / ON` opt-in toggle, `+ Add .MP4`, `+ Record Cam/Clip`; `#1–#40` tools: `+ Intro (2s)`, `+ Outro CTA`, `+ Lower-Third`, `+ Step ①②③`, `+ Spotlight`, `Cut/Crossfade/Fade-Through`, `Crisp Dev/Warm Face/Cyberpunk/Keynote` LUTs, `Cursor Boost`, `Cover .PNG`, `Chapter MP4 Splitter`, `Talking Head`, `Hide Cam 3s`, `+ 3D Tilt`, `Freeze 2s`, `+ Progress`, `+ CLI Box`, `+ Pro-Tip`, `Zoom Steps`, `Podcast Voice Master EQ`, `SRT/VTT/TXT/MD Bundle`, `+ Quiz Card`, `+ Key ⌘K`, `Before/After`, `+ Arrow ➔`, `60s Reel Cut`, `Jump-Cut Zoom`, `Pace 1.15x All`, `+ Lo-Fi Ambient Pad`, `+ Censor Bleep`, `3-Format Batch Export`, `Save/Apply Brand Kit`, `Auto-Hook 🔥`, `Chapter Cards`, `Silences 3x`, `Portrait Cam`, `Safe Guide`, `Loop Reel ↺`, `+ SFX Whoosh/Ding`, `CLI Cheat-Sheet .MD`, `+ Link Card 🔗`, `+ QR / Docs 📱`, `iPhone Bezel`, `MacBook Bezel`, `Copy Clickable SOP & Chapters (.MD)`) |
| **Studio Autopilot, Audacity-Grade Audio DSP, Viral/Coding Captions, Camera & Voiceover Manager** | `crates/audio/src/voice.rs` (`with_transparent_denoise`), `crates/audio/src/voice_level.rs` (`studio_mastering_chain`: 80Hz high-pass, adaptive gate, +2dB @ 250Hz warmth EQ, +3dB @ 5kHz presence EQ, 6.8kHz de-esser, -14 LUFS compressor/limiter; `VoiceProfile::broadcast_filter`), `apps/desktop-gpui/src/editor_tabs.rs` & `editor_window.rs` (`apply_studio_voice_profile`: `Podcast Studio`, `Tech Screencast`, `Noisy Room Fix`; `attach_camera_video_clicked` `+ Attach Cam (.mp4)` & `record_new_clip_from_editor` `+ Record Cam/Clip` in always-enabled `Camera` tab; `magic_polish_clicked`, `optimize_captions_caps_and_punctuation`, `CAPTION_PRESETS` including `hormozi-fire`, `terminal-cli`, `mrbeast-gold`, `glass-pill`, `editorial-serif`, `dev-neon`, `FONT_OPTIONS` with `JetBrains Mono / SF Mono`, `Impact / Heavy Display`, `Inter / Crisp UI`; `toggle_voiceover_recording`, `rerecord_audio_segment_at`, `delete_audio_segment_at`, `clear_all_audio_segments`, `add_text_at_playhead`, `Mute Original Clip Audio`; Camera presets `Small 25%`, `Med 40%`, `Lg Square 60%`, `Split 50/50`; Audio `Import Audio (.mp3/.wav)`, `Music Library`, `Background Music Volume` presets, `Auto-Ducking (-12 dB)`), `apps/desktop-gpui/src/teleprompter_window.rs` (`Slow`/`Med`/`Fast` 1-click WPM presets), `apps/desktop-gpui/src/transcription.rs` (`regroup_captions_for_social_reels`, `plan_filler_word_cuts`, `plan_silence_cuts`, `plan_pii_redaction_ranges`, `generate_youtube_chapters`, `generate_markdown_sop`) |
| **4K/60fps & Reels/Shorts Export + Clipboard Fix** | `apps/desktop-gpui/src/editor_export.rs` (defaults: `4K`, `60fps`, `Maximum` 0.45 bpp; `Export for Reels & Shorts (1080×1920 · 60fps)` at 0.12 bpp ~14.2 Mbps; `Copy Reels & Shorts MP4 to Clipboard`; `Copy PR / Slack GIF`; clipboard exports saved in `<recordings_dir>/Exports/`), `crates/enc-ffmpeg/src/h264.rs` (CRF 16 + H.264 `high` profile on `h264_videotoolbox`), `apps/desktop-gpui/src/platform.rs` (`copy_file_to_clipboard` sets `NSURL`, `NSFilenamesPboardType`, and `com.compuserve.gif`) |
| **Fast Dev Launch, GitHub Releases In-App Auto-Updater & macOS DMG Release Pipeline (`v0.6.5`)** | `start.sh` & `stop.sh` (`target/.cap-gpui-patch-stamp` skips `prepare-gpui-dependency.sh`, builds `--bin sricap-rec` with `split-debuginfo = "unpacked"` & `codegen-units = 256`), `apps/desktop-gpui/src/installer.rs` & `src/updates.rs` (`GITHUB_RELEASES_LATEST` `https://api.github.com/repos/srinivaskona7/Cap/releases/latest`, `parse_github_release`, `.zip` download & `ditto -x -k` hot-replacement of `/Applications/sricap-rec.app`), `scripts/build-gpui-binary.sh`, `scripts/bundle-gpui.mjs` (auto-builds `cap-muxer` & `cap-cli` sidecars via `build-desktop-binaries.mjs` if missing), `.github/workflows/release-sricap-mac.yml` (`mozilla-actions/sccache-action@v0.0.7`, parallel sidecar + GPUI compile, `--skip-build --bundles app`, custom `.VolumeIcon.icns`, Finder-positioned `sricap-rec.app` & `/Applications` drag-and-drop `UDZO` DMG) |
| **Multi-Product Open-Source Roadmap (Android Mirroring, Studio Voice & Neural Studio)** | `memory/OPENSOURCE_ROADMAP.md` (`Genymobile/scrcpy`, `sharpjs/adb_client`, `barry-ran/scrcpy-rs`, `jneem/nnnoiseless` pure Rust RNN speech noise reduction, `Datadog/dtln-rs`, `whisper-rs` Metal speech-to-text, Apple Vision face landmarks & gaze alignment) |
| **Web Platform & Chrome Extension Features** | `packages/database/types/metadata.ts`, `apps/web/lib/edit-transcript.ts`, `apps/web/lib/video-sop-and-bug-report.ts`, `apps/web/lib/viewer-retention.ts`, `apps/web/lib/workspace-semantic-search.ts`, `apps/chrome-extension/src/content/blur-overlay.tsx`, `apps/chrome-extension/src/shared/bug-report.ts` |

## Fast File-Lookup Cheat Sheet (Zero-Search Direct Paths)

| Domain / Task | Direct File Paths |
|---|---|
| **GPUI Desktop Entry & Windows** | `apps/desktop-gpui/src/main.rs`, `src/app_windows.rs`, `src/main_window.rs`, `src/controls_window.rs`, `src/teleprompter_window.rs`, `src/camera_window.rs`, `src/target_overlay.rs` |
| **GPUI Studio Video Editor** | `apps/desktop-gpui/src/editor_window.rs`, `src/editor_canvas.rs`, `src/editor_timeline.rs`, `src/editor_sidebar.rs`, `src/editor_panels.rs`, `src/editor_clips.rs`, `src/editor_crop.rs`, `src/editor_edits.rs`, `src/editor_color.rs`, `src/editor_export.rs` |
| **GPUI Screenshot Editor** | `apps/desktop-gpui/src/screenshot_editor.rs`, `src/screenshot_annotations.rs`, `src/screenshot_crop.rs`, `src/screenshot_export.rs` |
| **GPUI Recording, Upload & AI Captions** | `apps/desktop-gpui/src/recording.rs`, `src/upload.rs`, `src/transcription.rs`, `src/devices.rs`, `src/store.rs`, `src/settings_pages.rs` |
| **GPUI Hot-Loop Kernels (`opt-level = 3`)** | `apps/desktop-gpui/kernels/src/lib.rs` |
| **Rust Recording Actors (Instant & Studio)** | `crates/recording/src/lib.rs`, `src/instant_recording.rs`, `src/studio_recording.rs`, `src/recovery.rs`, `src/track_heal.rs`, `src/sync_calibration.rs` |
| **Rust `.cap` Project Config** | `crates/project/src/lib.rs`, `src/configuration.rs`, `src/meta.rs`, `src/cursor.rs` |
| **Rust `wgpu` Compositor & Shaders** | `crates/rendering/src/lib.rs`, `src/frame_pipeline.rs`, `src/zoom.rs`, `src/zoom_spring.rs`, `src/spring_mass_damper.rs`, `src/layers/*.rs` |
| **Rust Editor & Export Engine** | `crates/editor/src/lib.rs`, `crates/export/src/lib.rs`, `crates/cap-muxer/src/main.rs` |
| **Database Schema & Metadata Types** | `packages/database/schema.ts` (47 MySQL tables), `packages/database/types/metadata.ts` (`VideoMetadata`, `VideoCallToAction`, `VideoEditSpec`) |
| **Effect-TS Domain & Contracts** | `packages/web-domain/src/*.ts`, `packages/web-api-contract-effect/src/*.ts`, `packages/web-api-contract/src/*.ts` |
| **Effect-TS Backend Services** | `packages/web-backend/src/Videos/`, `src/Organisations/`, `src/Spaces/`, `src/Folders/`, `src/Comments/`, `src/S3Buckets/`, `src/Storage/`, `src/Tinybird/`, `src/AgentManagement.ts` |
| **Next.js 16 Web App Routes** | `apps/web/app/(org)/dashboard/`, `apps/web/app/s/[videoId]/`, `apps/web/app/c/[collectionId]/`, `apps/web/app/embed/`, `apps/web/app/api/`, `apps/web/app/mcp/` |
| **Media Server & Tov Codec** | `apps/media-server/src/`, `capcodec/src/` |

## Detailed Memory Files

Consult the topic-specific memory files in `memory/` before making changes in that area:

1. **[`memory/architecture.md`](./architecture.md)** — Complete monorepo map (`apps/*`, `packages/*`, `crates/*`, `capcodec/`, `vendor/`, `infra/`), full directory breakdown, data flow between desktop and web, and licensing boundaries (AGPLv3 vs MIT).
2. **[`memory/rust-media-engine.md`](./rust-media-engine.md)** — Deep dive into `crates/*`: recording actors (`kameo`), Instant vs Studio pipelines, `scap-*` platform capture, `wgpu` rendering/compositing layers, hardware encoders, `cap-muxer` crash resilience, and `cap-verify` deterministic testing.
3. **[`memory/desktop-apps.md`](./desktop-apps.md)** — Architecture and module-by-module map of `apps/desktop-gpui` (shipping GPUI native app, standalone Cargo workspace, `kernels/` opt-level split, Studio & Screenshot editors) vs `apps/desktop` (Tauri v2 + SolidStart dev harness), local Whisper/Parakeet transcription, and platform patches.
4. **[`memory/web-and-backend.md`](./web-and-backend.md)** — Next.js 16 + Effect-TS (`@effect/platform`, `@effect/rpc`, `Layer` DI) backend patterns, all 47 `@cap/database` Drizzle MySQL tables, `@cap/s3`, `apps/media-server` (`capcodec` / Tov), MCP/Agent APIs, Developer Platform, and Tinybird analytics.
5. **[`memory/workflows-and-invariants.md`](./workflows-and-invariants.md)** — Mandatory CI invariants (zero-comment rule, forbidden hand-edits on generated files, denied Clippy lints, Biome tab/double-quote rules), build/test commands, post-edit verification gates, and cross-agent operational policies.

## Cross-Agent Compatibility Map & Universal Bootstrap Standard

| AI Agent | Root Entry File | Subdirectory Entry | Memory Resolution |
|---|---|---|---|
| **Codex** | `AGENTS.md` | `apps/web/AGENTS.md` | Linked from `AGENTS.md` → `memory/MEMORY.md` |
| **Claude Code** | `CLAUDE.md` | `apps/web/CLAUDE.md` | `@AGENTS.md` and `@memory/MEMORY.md` imports |
| **Gemini CLI** | `GEMINI.md` | `apps/web/GEMINI.md` | `@AGENTS.md`, `@memory/MEMORY.md`, and private workspace memory |

### Universal Project Memory Bootstrap Standard
For every repository or new project worked on by Claude Code, OpenAI Codex, or Gemini CLI:
1. Maintain root `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` synchronized to `memory/MEMORY.md`.
2. Record exact module trees, file paths, database schemas, and pre-generation lint/format invariants in `memory/*.md` so agents never waste tokens or turns running exploratory searches.
3. Perform unrestricted web searches when researching external docs/APIs, and directly allowlist any blocked documentation or package domains so execution never stalls.

### Mandatory Git Push & GitHub Actions / Pages Engineering Rules

#### 1. Mandatory Git Push Protocol: SSH Only & Zero `gh auth`
1. **Standard Git over SSH Only**:
   - Always push using standard git over SSH: `git push origin <branch>` (e.g. `git push origin feature1` or `git push origin main`).
   - NEVER use `gh auth login`, `gh auth status`, or GitHub CLI tokens. Authentication on this machine is pre-configured via SSH (`git@github.com:...`).
2. **Handle Non-Fast-Forward Rejections**:
   - If a push is rejected because the remote has new commits, run `git pull --rebase origin <branch>` and then `git push origin <branch>`.
3. **Verify SSH Remote URL**:
   - Ensure the remote is set to SSH (`git remote -v`). If it starts with `https://`, immediately convert it: `git remote set-url origin git@github.com:srinivaskona7/Cap.git`.
4. **Correct Author Identity**:
   - Always commit as `git config user.email "kona.srinivas.app@gmail.com"` and `git config user.name "kona_srinivas"` for public repos. NEVER use `@apple.com`.

#### 2. GitHub Actions & GitHub Pages Deployment Standards
- **Pages Source Setting**: In repo Settings → Pages, ensure Source is set to "GitHub Actions" when using `.github/workflows/pages.yml`.
- **LTS Runner & Modern Actions**: Pin `runs-on: ubuntu-24.04` and use `actions/configure-pages@v6`, `actions/upload-pages-artifact@v5`, `actions/deploy-pages@v5`.
- **Include `.nojekyll`**: Always add `run: touch <dist-dir>/.nojekyll` before artifact upload to prevent Jekyll from omitting assets or dotfolders.
- **Fast Deploy Pipeline (< 1 min)**: Separate deployment from heavy testing. Never install full browser packages (`npx playwright install --with-deps`) or run large E2E smoke suites in `pages.yml`. Keep heavy test suites in PR CI (`ci.yml`).
- **Clean CI Runner Test Readiness**: When tests depend on generated manifests or catalogs, configure `npm test` to emit them first so a freshly cloned CI runner never crashes with `ERR_MODULE_NOT_FOUND`.
- **Root-Anchored `.gitignore`**: Always use leading slashes for test and build artifact ignore rules (e.g. `/coverage/`, not `coverage/`) so valid source directories (such as `src/features/coverage/`) are never ignored by Git.

