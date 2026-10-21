# Local Setup, Launch & Testing Guide (`local-run.md`)

This guide covers everything needed to install, run, unlock Pro features, and test the **Cap** Desktop App (`apps/desktop-gpui`), Web Platform (`apps/web`), Chrome Extension, and MCP Server locally.

---

## 1. Prerequisites (One-Time Setup)

Ensure the following tools are installed on macOS:
- **Node.js** `>= 20`
- **Bun** `1.4.0`
- **Rust** (`rustup` / `cargo`, `1.88+`)
- **Docker Desktop** (for local MySQL + MinIO S3 storage)

```bash
# Install Bun (if not installed)
curl -fsSL https://bun.sh/install | bash

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 2. Initial Repository Bootstrap

Run these commands once from the repository root:

```bash
# 1. Install all workspace dependencies
bun install

# 2. Generate local .env configuration
bun run env-setup

# 3. Download native FFmpeg & ONNX Runtime libraries for desktop recording/transcription
bun run cap-setup
```

---

## 3. Launch the Local Web Platform (`http://localhost:3000`)

The web platform powers Instant Mode share links (`/s/[videoId]`), Web Transcript Editing (`/s/[videoId]/edit`), Step-by-Step SOP & Bug Bundles, Viewer Retention Heatmaps, and Cross-Workspace Semantic Search.

```bash
# 1. Start MySQL and MinIO (local S3) in Docker
bun run docker:up

# 2. Push Drizzle database schema to local MySQL
bun run db:push

# 3. Start the Next.js web server
bun run dev:web
```

### How to Unlock Cap Pro / Max Features Locally (Free in Dev)
1. In a separate terminal, open Drizzle Studio:
   ```bash
   bun run db:studio
   ```
2. Open the **`users`** table.
3. Find your signed-in user row and set **`stripeSubscriptionStatus`** to **`active`**.
4. Refresh `http://localhost:3000` — all Pro AI features are now unlocked and the share player watermark overlay is removed.

---

## 4. Launch the Native Desktop App (`apps/desktop-gpui`)

### Option A: Run Directly in Dev Mode (Recommended)
```bash
cargo run --manifest-path apps/desktop-gpui/Cargo.toml --bin cap-gpui
```
> **macOS Permissions Note**: Grant **Screen Recording**, **Microphone**, and **Camera** permissions to the Terminal or IDE running `cargo run` when prompted by macOS.

### Option B: Build a Standalone macOS `Cap.app` Bundle
```bash
# Apple Silicon (M1 / M2 / M3 / M4)
bun run gpui:build aarch64-apple-darwin

# Intel Mac
bun run gpui:build x86_64-apple-darwin
```
The packaged app bundle is output to:
`apps/desktop/src-tauri/target/<target>/release/bundle/macos/Cap.app`

---

## 5. How to Test the Tier 1 & Tier 2 Features Locally

### A. Automated Unit Tests
```bash
# Web Unit Tests (Vitest)
cd apps/web
bun run test __tests__/unit/tier1-and-tier2-features.test.ts __tests__/unit/edit-transcript.test.ts __tests__/unit/mcp-server.test.ts

# Desktop GPUI Studio Unit Tests (Rust)
cargo test --manifest-path apps/desktop-gpui/Cargo.toml studio_plans_filler_and_silence_cuts_and_redacts_pii_and_builds_sop
```

### B. Desktop Studio & Screenshot Editor (`cap-gpui`)
1. **#1 One-Click Filler Word & Silence Removal**:
   - Record a Studio clip, open the **Captions** tab in the right sidebar, and click **Generate Captions**.
   - Under **Smart Edit, Redact & SOP**, click **Remove Fillers (N)** or **Remove Silences (N)** to automatically split and ripple-delete those ranges on the timeline.
2. **#2 Transcript-Driven Video Editing (Delete Text to Cut Video)**:
   - In the **Captions** tab under **Smart Edit, Redact & SOP**, click **Cut** next to any transcript line to ripple-delete that spoken segment from the video.
3. **#3 Video-to-SOP Step-by-Step Guide Generator**:
   - Click **Copy SOP Guide** in the **Captions** tab to generate a Markdown SOP combining your recorded cursor click coordinates (`x%, y%`), timestamps, and spoken instructions, copied directly to your clipboard.
4. **#4 Automated PII & Secret Auto-Redaction (Smart Blur)**:
   - **In Studio Editor**: Click **Auto-Redact PII** in the **Captions** tab to replace spoken emails/keys/passwords with `[REDACTED]` and insert keyframed `Sensitive` blur masks on the Mask track.
   - **In Screenshot Editor**: Take a screenshot and click **Auto-Redact PII** in the top annotation toolbar to automatically place blur masks over sensitive text bands.

### C. Web Platform (`http://localhost:3000`)
1. **#1, #2, #4 Web Transcript Editor (`/s/<videoId>/edit`)**:
   - Use **Remove fillers**, **Remove silences**, and **Redact PII** pills in the Transcript header.
   - Hover any paragraph timestamp and click **Cut** (or drag-select words and click **Delete**) to cut the video by editing text.
2. **#3 & #5 SOP Guide & Developer Bug-Report Bundle (`/s/<videoId>` → Summary Tab)**:
   - Click **Generate SOP & Bug Bundle**.
   - Click any step to seek the video, click **Copy Markdown / Notion**, or use **Create Linear Issue**, **Create GitHub Issue**, or **Create Jira Issue** for 1-click bug filing.
3. **#6 Viewer Retention Heatmaps & Per-Prospect Watch Alerts (`/s/<videoId>` → Activity Tab)**:
   - Click **Retention & Watch Alerts** in the top analytics bar.
   - Generate a tracked link (`?prospect=lead@company.com`), open it in an Incognito window, watch/rewatch segments, and inspect the 20-bar **Viewer Retention Heatmap** and **Recent Prospect & Viewer Sessions**.
4. **#7 Cross-Workspace Semantic AI Search (`/dashboard/caps` & MCP)**:
   - Use the **Ask My Team's Videos** search bar at the top of `/dashboard/caps` to search across spoken words, summaries, chapters, SOP steps, and OCR text across your workspace.
   - Or query the MCP tool `caps_ask_workspace` at `POST http://localhost:3000/api/mcp`.

---

## 6. Git Branch Workflow (`feature1` vs `main`)

All custom feature work lives on the **`feature1`** branch so **`main`** stays clean for pulling upstream updates:

```bash
# Check current branch
git branch --show-current   # feature1

# Sync latest upstream changes into main and rebase feature1
git checkout main
git pull origin main
git checkout feature1
git rebase main
```
