# Desktop Applications (`apps/desktop-gpui` & `apps/desktop`)

Cap maintains two desktop application directories in the monorepo that share the same underlying Rust crates (`cap-recording`, `cap-editor`, `cap-rendering`, `cap-export`, `cap-project`):

---

## 1. `apps/desktop-gpui` — Primary Shipping Desktop App (`sricap-rec`)

- **Status**: **Active production desktop client** (`sricap-rec`, v0.6.1+, bundle ID `com.sri.sricap-rec`).
- **Standalone Cargo Workspace**:
  - Intentionally excluded from the root `Cargo.toml` workspace (`apps/desktop-gpui/Cargo.toml` defines its own `[workspace]` with members `[".", "kernels"]` and `[[bin]] name = "sricap-rec"`).
  - **Why**: GPUI (`wingleeio/zed` fork) pulls a massive dependency tree including `wgpu` v29, whereas `cap-rendering` uses `wgpu` v25 with `[patch.crates-io]` `vendor/wgpu-hal`. Isolating the workspace keeps `Cargo.lock`, `cargo hakari`, and Tauri patches cleanly separated.
- **Performance Architecture (`kernels/` & Fast Dev Profile)**:
  - In dev builds (`[profile.dev]`), `cap-desktop-gpui` compiles at `opt-level = 0` with `incremental = true`, `codegen-units = 256`, and `split-debuginfo = "unpacked"` (skipping macOS `dsymutil` on incremental links), while all dependencies compile at `opt-level = 2` and `apps/desktop-gpui/kernels` (`cap-gpui-kernels`) compiles at `opt-level = 3` so hot pixel/audio loops (`frame_image`, waveform/sample processing) take ~1ms instead of ~42ms.
  - In release builds (`[profile.release]`), `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `panic = "abort"`, and `strip = true` keep the final `.dmg` compact (~40–50MB).
  - `./start.sh` caches the SHA-256 of `apps/desktop-gpui/patches/*.patch` in `target/.cap-gpui-patch-stamp` so `scripts/prepare-gpui-dependency.sh` is skipped when patches are unchanged.
- **Packaging & Sidecar Invariant (`scripts/bundle-gpui.mjs`)**:
  - `tauri.conf.json` bundles `../desktop/src-tauri/binaries/cap-muxer` and `../desktop/src-tauri/binaries/cap-cli`.
  - `scripts/bundle-gpui.mjs` automatically builds `scripts/build-desktop-binaries.mjs <target>` if `cap-muxer-<target>` or `cap-cli-<target>` is missing, ensures `.env` exists at repo root, defaults to `--bundles app,dmg` when `TAURI_SIGNING_PRIVATE_KEY` is unset, and copies the finished `sricap-rec.app` and `sricap-rec.dmg` into `apps/desktop/src-tauri/target/release/bundle/macos/`.
- **Complete Module Map (`apps/desktop-gpui/src/`)**:
  - **Core App & Window Management**:
    - `main.rs`, `app_windows.rs`, `main_window.rs`, `mode_select_window.rs`, `controls_window.rs`, `tray.rs`, `menus.rs`, `hotkeys.rs`, `single_instance.rs`, `deeplink.rs`, `updates.rs`, `installer.rs`.
  - **Recording, Target Picker, Camera & Teleprompter**:
    - `recording.rs`, `session.rs`, `target_overlay.rs`, `target_thumbnails.rs`, `devices.rs`, `camera_window.rs`, `camera_loading.rs`, `camera_blur.rs`, `camera_blur_portable.rs`, `teleprompter_window.rs`, `script_text.rs`, `doodle.rs`.
  - **Studio Video Editor**:
    - `editor_window.rs` (main coordinator), `editor_canvas.rs` (zero-copy `paint_surface` preview & gizmos), `editor_timeline.rs` (multi-track timeline), `editor_sidebar.rs`, `editor_panels.rs`, `editor_tabs.rs`, `editor_clips.rs`, `editor_crop.rs`, `editor_edits.rs`, `editor_color.rs`, `editor_audio.rs`, `editor_export.rs`, `editor_preparing.rs`, `editor_modal.rs`, `presets.rs`.
  - **Screenshot Capture & Editor**:
    - `screenshot.rs`, `screenshot_editor.rs`, `screenshot_annotations.rs` (arrows, shapes, text, redact/blur, numbering via `tiny-skia` + `cosmic-text`), `screenshot_crop.rs`, `screenshot_export.rs`.
  - **AI Transcription, Upload, Library & Settings**:
    - `transcription.rs` (local Whisper via `whisper-rs` and Parakeet via `parakeet-rs`), `upload.rs` (progressive & multipart S3 upload), `library.rs`, `import.rs`, `feeds.rs`, `store.rs`, `settings_window.rs`, `settings_pages.rs`, `auth.rs`, `sign_in.rs`.
  - **Permissions, Onboarding & Diagnostics**:
    - `permissions.rs`, `permissions_ui.rs`, `permission_guide.rs`, `onboarding_window.rs`, `diagnostics.rs`, `dev_restore.rs`, `platform.rs`.
- **Build Command**: `bun run gpui:build <target>` (`scripts/bundle-gpui.mjs`).

---

## 2. `apps/desktop` — Legacy Tauri v2 App & Dev Harness

- **Status**: Last shipped as Cap in v0.6.0; retained in the root workspace as a development harness and reference implementation.
- **Stack**:
  - Frontend: **SolidStart** + TypeScript + Tailwind CSS (`apps/desktop/src/`) + `@cap/ui-solid`.
  - Backend: **Tauri v2** (`apps/desktop/src-tauri/`) in the root Cargo workspace.
  - Type-Safe IPC: Uses `specta` + `tauri-specta` to generate `apps/desktop/src/utils/tauri.ts` on debug runs.
- **Critical File Rules for `apps/desktop`**:
  - **NEVER hand-edit** `apps/desktop/src/utils/tauri.ts`, `apps/desktop/src-tauri/gen/**`, or `packages/ui-solid/src/auto-imports.d.ts`.
  - `apps/desktop/src/utils/queries.ts` **IS hand-written** and should be edited normally.
  - Biome a11y lint rules are disabled under `apps/desktop/**` (`biome.json`), but enforced in `apps/web` and `packages/ui`.
- **Shared Storage Compatibility**:
  - Both `apps/desktop-gpui` and `apps/desktop` read the same settings store and recordings library on disk, and use separate rolling log prefixes (`src/diagnostics.rs`) so they never corrupt each other's logs.
