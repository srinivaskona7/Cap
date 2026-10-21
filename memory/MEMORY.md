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
| **Offline Pro Unlock & Local Storage Controls** | `apps/desktop-gpui/src/store.rs` (`AuthSnapshot::is_upgraded() -> true`), `apps/desktop-gpui/src/settings_window.rs` (`has_cap_pro = true`, custom recordings folder picker, storage used banner, `Delete All Local Recordings`), `apps/desktop-gpui/src/library.rs` (`compute_directory_size_bytes`, `delete_all_recordings`) |
| **Google Meet-Style Browser Tabs & Windows Picker** | `apps/desktop-gpui/src/devices.rs` (`BrowserTabOption`, `list_browser_tabs_for_windows`, `activate_browser_tab` via AppleScript across Chrome, Arc, Brave, Edge, Safari + fallback window IDs), `apps/desktop-gpui/src/main_window.rs` (`browser_tabs_mode`, `[All Windows] | [Browser Tabs]` toggle in `render_target_grid`, clicking `Window` tile opens picker panel directly) |
| **9:16 Vertical Teaching Studio & Multi-Image Sequence Manager** | `apps/desktop-gpui/src/editor_sidebar.rs` (`render_social_layout_presets`, `apply_tech_reel_preset`: 3-zone 1080×1920 layout with Top Camera at `y=0.17`, Center Framed MacOS/Browser Window at `y=0.53`, Bottom Safe Zone Hormozi/Dev-Neon Captions at `y=0.80`; `Terminal Dev Reel (9:16)` & `Web Demo Reel (9:16)` presets; `Upload Images (2s Default)` multi-file picker, numbered `#1, #2...` sequence cards with `↑/↓` reorder, `1s/2s/3s/5s/Full` duration buttons, `Center/TL/TR/BL/BR` placement, and canvas drag focus; `Add Reel Hook Banner`) |
| **Studio Autopilot, Live Voiceover Manager, Camera & Audio** | `apps/desktop-gpui/src/editor_tabs.rs` & `editor_window.rs` (`magic_polish_clicked`, `toggle_voiceover_recording`, `rerecord_audio_segment_at`, `delete_audio_segment_at`, `clear_all_audio_segments`, `add_text_at_playhead`, `Mute Original Clip Audio (Keep Voiceover/Music)`; Camera presets `Small 25%`, `Med 40%`, `Lg Square 60%`, `Split 50/50`; Audio `Import Audio (.mp3/.wav)`, `Music Library`, `Background Music Volume` presets, `Auto-Ducking (-12 dB)`), `apps/desktop-gpui/src/transcription.rs` (`regroup_captions_for_social_reels`, `plan_filler_word_cuts`, `plan_silence_cuts`, `plan_pii_redaction_ranges`, `generate_youtube_chapters`, `generate_markdown_sop`) |
| **4K/60fps & Reels/Shorts Export + Clipboard Fix** | `apps/desktop-gpui/src/editor_export.rs` (defaults: `4K`, `60fps`, `Maximum` 0.45 bpp; `Export for Reels & Shorts (1080×1920 · 60fps)` at 0.12 bpp ~14.2 Mbps; `Copy Reels & Shorts MP4 to Clipboard`; `Copy PR / Slack GIF`; clipboard exports saved in `<recordings_dir>/Exports/`), `crates/enc-ffmpeg/src/h264.rs` (CRF 16 + H.264 `high` profile on `h264_videotoolbox`), `apps/desktop-gpui/src/platform.rs` (`copy_file_to_clipboard` sets `NSURL`, `NSFilenamesPboardType`, and `com.compuserve.gif`) |
| **Fast Dev Launch & macOS DMG Release Pipeline** | `start.sh` & `stop.sh` (`target/.cap-gpui-patch-stamp` skips `prepare-gpui-dependency.sh`, builds `--bin sricap-rec` with `split-debuginfo = "unpacked"` & `codegen-units = 256`), `scripts/build-gpui-binary.sh`, `scripts/bundle-gpui.mjs` (auto-builds `cap-muxer` & `cap-cli` sidecars via `build-desktop-binaries.mjs` if missing, defaults to `--bundles app,dmg` when unsigned), `.github/workflows/release-sricap-mac.yml` |
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

