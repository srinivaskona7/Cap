# Cap Repository Memory Index

This directory contains structured, long-lived architectural and operational memory for the **Cap** (`CapSoftware/Cap`) monorepo, designed to work seamlessly across **OpenAI Codex**, **Claude Code**, and **Google Gemini CLI** with zero token waste and zero redundant discovery turns.

## Quick Orientation

- **Product**: Open-source screen recording, studio editing, screenshot annotation & async video messaging platform (Instant Mode + Studio Mode + Screenshot Mode + Self-Hostable Cloud).
- **Monorepo Stack**: Turborepo + Bun (`1.4.0`) + Node (`>=20`) + Rust (`1.88+`, Cargo workspaces).
- **Primary Shipping Desktop App**: `apps/desktop-gpui` (`cap-gpui`, pure Rust GPUI app in a standalone Cargo workspace).
- **Legacy Desktop / Dev Harness**: `apps/desktop` (Tauri v2 + SolidStart + `specta` IPC).
- **Web Platform**: `apps/web` (Next.js 16 App Router, React 19, Effect-TS, Drizzle ORM + MySQL/PlanetScale, NextAuth + WorkOS SSO, S3/CloudFront, Tinybird analytics, MCP Server & Developer Platform).
- **Media Engine**: `crates/*` (50+ Rust crates for screen capture, camera, audio, `wgpu` shader rendering, hardware encoding, out-of-process MP4 muxing, and deterministic A/V verification).

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

#### 1. Git Push, Author Email & SSH Remote Invariant (Mandatory)
- **Public Repo Git Author Email**: Always use `kona.srinivas.app@gmail.com` (`git config user.email "kona.srinivas.app@gmail.com"`) for public GitHub repositories. NEVER use `@apple.com` email addresses on public repos.
- **Always push with standard git over SSH**: Always execute `git push origin <branch>` (e.g. `git push origin main`).
- **Zero `gh auth` Friction**: Authentication on this machine is pre-configured via SSH (`git@github.com:...`). Never run `gh auth login`, `gh auth status`, or rely on GitHub CLI tokens to push code. Directly commit and push. Only investigate auth if `git push` returns an actual SSH permission error.
- **Ensure SSH Remote**: If any repo remote URL starts with HTTPS (`https://github.com/...`), immediately convert it to SSH: `git remote set-url origin git@github.com:<org-or-user>/<repo>.git`.

#### 2. GitHub Actions & GitHub Pages Deployment Standards
- **Pages Source Setting**: In repo Settings → Pages, ensure Source is set to "GitHub Actions" when using `.github/workflows/pages.yml`.
- **LTS Runner & Modern Actions**: Pin `runs-on: ubuntu-24.04` and use `actions/configure-pages@v6`, `actions/upload-pages-artifact@v5`, `actions/deploy-pages@v5`.
- **Include `.nojekyll`**: Always add `run: touch <dist-dir>/.nojekyll` before artifact upload to prevent Jekyll from omitting assets or dotfolders.
- **Fast Deploy Pipeline (< 1 min)**: Separate deployment from heavy testing. Never install full browser packages (`npx playwright install --with-deps`) or run large E2E smoke suites in `pages.yml`. Keep heavy test suites in PR CI (`ci.yml`).
- **Clean CI Runner Test Readiness**: When tests depend on generated manifests or catalogs, configure `npm test` to emit them first so a freshly cloned CI runner never crashes with `ERR_MODULE_NOT_FOUND`.
- **Root-Anchored `.gitignore`**: Always use leading slashes for test and build artifact ignore rules (e.g. `/coverage/`, not `coverage/`) so valid source directories (such as `src/features/coverage/`) are never ignored by Git.

