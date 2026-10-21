# Desktop Applications (`apps/desktop-gpui` & `apps/desktop`)

Cap maintains two desktop application directories in the monorepo that share the same underlying Rust crates (`cap-recording`, `cap-editor`, `cap-rendering`, `cap-export`, `cap-project`):

---

## 1. `apps/desktop-gpui` — Primary Shipping Desktop App

- **Status**: **Active production desktop client** (`cap-gpui`, v0.6.1+). Ships as Cap on macOS, Windows, and Linux.
- **Standalone Cargo Workspace**:
  - Intentionally excluded from the root `Cargo.toml` workspace (`apps/desktop-gpui/Cargo.toml` defines its own `[workspace]` with members `[".", "kernels"]`).
  - **Why**: GPUI (`wingleeio/zed` fork) pulls a massive dependency tree including `wgpu` v29, whereas `cap-rendering` uses `wgpu` v25 with `[patch.crates-io]` `vendor/wgpu-hal`. Isolating the workspace keeps `Cargo.lock`, `cargo hakari`, and Tauri patches cleanly separated.
- **Performance Architecture (`kernels/`)**:
  - In dev builds (`[profile.dev]`), `cap-desktop-gpui` compiles at `opt-level = 0` with `incremental = true` for fast rebuilds (~seconds), while all dependencies compile at `opt-level = 2` and `apps/desktop-gpui/kernels` (`cap-gpui-kernels`) compiles at `opt-level = 3` so hot pixel/audio loops (`frame_image`, waveform/sample processing) take ~1ms instead of ~42ms.
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
