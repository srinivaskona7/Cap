# Cap Monorepo Architecture & Complete Project Structure Map

## 1. System Overview

Cap is a polyglot monorepo combining native desktop screen recording, studio editing, and screenshot annotation (Rust + GPUI/Tauri + `wgpu`) with a cloud/self-hosted video sharing, AI, and collaboration platform (TypeScript + Next.js 16 + Effect-TS + Drizzle/MySQL + S3 + Tinybird).

### Three Core Capture & Editing Modes
1. **Instant Mode**: Encodes and uploads fragmented MP4/HLS segments progressively while recording (`crates/recording/src/instant_recording.rs`) with live provisional transcription (`transcription.live.json`) so a shareable link is ready the instant recording stops.
2. **Studio Mode**: Captures high-framerate display, camera, microphone, system audio, and high-frequency cursor/keyboard telemetry locally (`crates/recording/src/studio_recording.rs`), then opens the non-linear GPU editor (`cap-editor` + `cap-rendering` + `apps/desktop-gpui/src/editor_window.rs`) for custom backgrounds, spring-physics cursor zooms, 3D camera bubbles, masks/blurs, color grading, AI captions, and MP4/GIF export.
3. **Screenshot Mode**: Captures displays/windows/regions (`apps/desktop-gpui/src/screenshot.rs`) and opens the native GPUI screenshot editor (`screenshot_editor.rs`, `screenshot_annotations.rs`, `screenshot_crop.rs`, `screenshot_export.rs`) using `tiny-skia` and `cosmic-text` with instant cloud sharing.

---

## 2. Complete Directory & Module Map

```text
Cap/
├── AGENTS.md                        # Canonical cross-agent instructions (Codex, Claude, Gemini)
├── CLAUDE.md                        # Claude Code entry (@AGENTS.md + @memory/MEMORY.md)
├── GEMINI.md                        # Gemini CLI entry (@AGENTS.md + @memory/MEMORY.md)
├── Cargo.toml                       # Root Rust workspace (crates/*, apps/cli, apps/desktop/src-tauri, apps/render-farm)
├── biome.json                       # Biome formatter/linter (tabs, double quotes, organizeImports)
├── package.json                     # Bun 1.4.0 + Turborepo scripts & workspace definitions
├── turbo.json                       # Turborepo task pipeline
├── memory/                          # Shared cross-agent architectural memory
│   ├── MEMORY.md                    # Master memory index & zero-search file lookup table
│   ├── architecture.md              # Monorepo map & directory tree
│   ├── rust-media-engine.md         # crates/* media, capture, wgpu rendering & muxing guide
│   ├── desktop-apps.md              # apps/desktop-gpui & apps/desktop module breakdown
│   ├── web-and-backend.md           # apps/web, Effect-TS backend, & 47-table Drizzle schema
│   └── workflows-and-invariants.md  # CI rules, Clippy/Biome invariants, & universal agent policies
├── apps/
│   ├── desktop-gpui/                # Primary shipping native desktop app (standalone Cargo workspace)
│   │   ├── Cargo.toml               # Standalone workspace: [".", "kernels"]
│   │   ├── kernels/                 # opt-level=3 hot pixel/waveform loops (cap-gpui-kernels)
│   │   └── src/                     # GPUI windows, Studio editor, Screenshot editor, Whisper/Parakeet
│   ├── desktop/                     # Legacy Tauri v2 + SolidStart desktop app / dev harness
│   │   ├── src/                     # SolidStart UI (utils/queries.ts hand-written; utils/tauri.ts generated)
│   │   └── src-tauri/               # Tauri v2 Rust backend
│   ├── web/                         # Next.js 16 (App Router, Turbopack), React 19, Effect-TS web platform
│   │   ├── app/(org)/dashboard/     # Spaces, folders, videos, analytics, settings, billing, S3, SSO, Loom import
│   │   ├── app/s/[videoId]/         # Shareable video player, transcript, chapters, comments, reactions, CTA
│   │   ├── app/c/[collectionId]/    # Public folder collection pages
│   │   ├── app/embed/               # Embeddable video player routes
│   │   ├── app/mcp/                 # Model Context Protocol (MCP) OAuth & server routes
│   │   ├── app/messenger/           # Customer support messenger
│   │   └── app/api/                 # Effect-TS HttpApi & Next.js route handlers
│   ├── media-server/                # Bun + Hono + FFmpeg + capcodec server-side media processor
│   ├── render-farm/                 # Rust AWS Lambda / S3 cloud Studio renderer
│   ├── cli/                         # Rust CLI (cap-cli)
│   ├── mobile/                      # Expo / React Native mobile companion app
│   ├── chrome-extension/            # Browser tab/screen recording WebExtension
│   ├── discord-bot/                 # Cloudflare Workers Discord bot
│   └── status/                      # Status page service
├── packages/
│   ├── database/                    # Drizzle ORM MySQL schema (47 tables), migrations/, emails/, loops/
│   ├── web-domain/                  # Pure Effect domain entities, branded IDs, and domain errors
│   ├── web-api-contract/            # @ts-rest & Effect API contracts
│   ├── web-api-contract-effect/     # @effect/platform HttpApi & @effect/rpc contracts
│   ├── web-backend/                 # Effect-TS Layer services (Videos, Organisations, Spaces, Storage, AgentAuth, etc.)
│   ├── s3/                          # S3 & CloudFront presigned URL / multipart upload service
│   ├── env/                         # Zod environment validation (@t3-oss/env-nextjs)
│   ├── ui/                          # React 19 UI components
│   ├── ui-solid/                    # SolidJS UI components (auto-imports.d.ts is generated)
│   ├── recorder-core/               # Browser MediaStream / WebCodecs recording core
│   ├── sdk-embed/                   # Embeddable player SDK
│   ├── sdk-recorder/                # Embeddable recorder SDK
│   ├── utils/                       # Shared TS utilities
│   └── local-docker/                # Local MySQL / MinIO Docker orchestration
├── crates/                          # 50+ Rust crates (capture, audio, camera, rendering, encoding, muxing, verify)
├── capcodec/                        # Tov custom H.264 screen-content encoder snapshot & bundle
├── vendor/                          # Patched Rust crates (wgpu-hal v25, tao, tauri-nspanel)
├── scripts/                         # Build, bundle-gpui, analytics (Tinybird), and benchmark scripts
└── infra/                           # SST (sst.config.ts) cloud infrastructure
```

---

## 3. Applications (`apps/*`)

| Path | Package / Binary | Tech Stack | Purpose |
|---|---|---|---|
| `apps/desktop-gpui` | `cap-desktop-gpui` (`cap-gpui`) | Rust 2024, GPUI (Zed fork), `wgpu` 25/29, `tokio` | **Primary shipping desktop client** (macOS, Windows, Linux). Standalone Cargo workspace. |
| `apps/desktop` | `@cap/desktop` | Tauri v2, SolidStart, TypeScript, Rust (`src-tauri`) | **Legacy desktop client & dev harness** (last shipped in 0.6.0). Uses `specta` (`tauri.ts`) for IPC. |
| `apps/web` | `@cap/web` | Next.js 16 (Turbopack), React 19, Tailwind, Effect-TS | Marketing site, docs, user/team dashboard, video player, comments, AI features, MCP server, and HTTP/RPC API routes. |
| `apps/media-server` | `@cap/media-server` | Bun, Hono, `mediabunny`, `node-av`, FFmpeg, `capcodec` | Server-side video transcoding, muxing, audio extraction, and thumbnail/waveform generation. |
| `apps/render-farm` | `cap-render-farm` | Rust, AWS Lambda / S3 | Cloud rendering worker for Studio recordings. |
| `apps/cli` | `cap-cli` | Rust | Command-line interface for driving Cap recordings and exports. |
| `apps/mobile` | `@cap/mobile` | Expo, React Native | Mobile companion app for iOS/Android. |
| `apps/chrome-extension` | `@cap/chrome-extension` | TypeScript, WebExtension APIs | Browser tab/screen recording extension built on `@cap/recorder-core`. |
| `apps/discord-bot` | `@cap/discord-bot` | TypeScript, Cloudflare Workers | Community Discord bot integration. |
| `apps/status` | `@cap/status` | TypeScript | Status page service. |

---

## 4. Shared TypeScript Packages (`packages/*`)

| Package | Name | Role |
|---|---|---|
| `packages/database` | `@cap/database` | Drizzle ORM MySQL/PlanetScale schema (`schema.ts`, 47 tables), migrations (`migrations/`), NextAuth (`auth-options.ts`), WorkOS SSO, Loops contact sync, and React Email templates (`emails/`). |
| `packages/web-domain` | `@cap/web-domain` | Pure domain entities (`Video`, `User`, `Organisation`, `Space`, `Comment`, `Folder`, `S3Bucket`, `PublicCollection`, `Agent`, `Storage`), branded IDs, and Effect domain errors. |
| `packages/web-api-contract` | `@cap/web-api-contract` | Shared API contracts (`@ts-rest` / Effect schemas). |
| `packages/web-api-contract-effect` | `@cap/web-api-contract-effect` | `@effect/platform` `HttpApi` and `@effect/rpc` contract definitions. |
| `packages/web-backend` | `@cap/web-backend` | Effect-TS `Layer` services (`Videos`, `S3Buckets`, `Storage`, `Spaces`, `Organisations`, `Users`, `Folders`, `Comments`, `Tinybird`, `AgentAuth`, `AgentManagement`) implementing business logic against `@cap/database` and `@cap/s3`. |
| `packages/s3` | `@cap/s3` | S3 & CloudFront presigned URLs, multipart uploads, and custom S3-compatible bucket abstraction. |
| `packages/env` | `@cap/env` | Centralized `@t3-oss/env-nextjs` / Zod environment validation for server and client variables. |
| `packages/ui` | `@cap/ui` | Shared React 19 UI component library (used by `apps/web`). |
| `packages/ui-solid` | `@cap/ui-solid` | Shared SolidJS UI component library (used by `apps/desktop`). |
| `packages/recorder-core` | `@cap/recorder-core` | Browser-based MediaStream / WebCodecs recording core. |
| `packages/sdk-embed` | `@cap/sdk-embed` | Embeddable Cap player SDK. |
| `packages/sdk-recorder` | `@cap/sdk-recorder` | Embeddable Cap recorder SDK. |
| `packages/utils` | `@cap/utils` | Shared TypeScript utilities and helpers. |
| `packages/local-docker` | `@cap/local-docker` | Local development Docker Compose orchestration (MySQL, MinIO, etc.). |
| `packages/config` & `packages/tsconfig` | — | Shared Biome/TypeScript configurations. |

---

## 5. Custom Codec, Vendor Patches, & Infra

- **`capcodec/`**: Source snapshot and Git bundle (`capcodec.bundle`) for **Tov**, Cap's custom H.264 screen-content encoder used by `apps/media-server` when `CAP_MEDIA_VIDEO_ENCODER=capcodec`.
- **`vendor/`**: Patched upstream Rust crates required for desktop stability:
  - `vendor/wgpu-hal`: Patched `wgpu-hal` v25 for GPU surface/texture interop.
  - `vendor/tao`: Patched windowing library for Tauri v2.
  - `vendor/tauri-nspanel`: Fixes Objective-C `NSWindow` over-release and `dealloc` crash on macOS.
- **`infra/`**: SST (`sst.config.ts`) infrastructure-as-code definitions.

---

## 6. Licensing Architecture

- **MIT License (`licenses/LICENSE-MIT`)**: All crates in the `cap-camera*` (`crates/camera*`) and `scap-*` (`crates/scap-*`) families.
- **AGPLv3 (`LICENSE`)**: All other applications, packages, and crates in the repository.
