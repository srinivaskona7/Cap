#!/usr/bin/env bash
set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 1. Stop any running instance quietly
"$REPO_ROOT/stop.sh" --quiet 2>/dev/null || true

# 2. Ensure tools and shims are in PATH
export PATH="$HOME/bin:$HOME/.cargo/bin:$HOME/.bun/bin:/opt/homebrew/bin:$PATH"
export RUST_LOG="${RUST_LOG:-warn,cap_desktop_gpui=info,cap_recording=info,cap_export=info}"

# 3. Fast patch-stamp check (stored in target/ so ../zed-cap stays clean)
ZED_DIR="$REPO_ROOT/../zed-cap"
rm -f "$ZED_DIR/.cap-gpui-patch-stamp" 2>/dev/null || true
mkdir -p "$REPO_ROOT/target"
STAMP_FILE="$REPO_ROOT/target/.cap-gpui-patch-stamp"
CURRENT_STAMP="$(shasum -a 256 "$REPO_ROOT"/apps/desktop-gpui/patches/*.patch 2>/dev/null | shasum -a 256 | awk '{print $1}')"

if [[ ! -d "$ZED_DIR/.git" ]] || [[ ! -f "$STAMP_FILE" ]] || [[ "$(cat "$STAMP_FILE" 2>/dev/null)" != "$CURRENT_STAMP" ]]; then
	echo "🔄 Syncing GPUI patches..."
	bash "$REPO_ROOT/scripts/prepare-gpui-dependency.sh"
	echo "$CURRENT_STAMP" > "$STAMP_FILE"
fi

cd "$REPO_ROOT"
SECONDS=0
if [[ "$1" == "--release" ]]; then
	echo "🔨 Building Cap GPUI (release)..."
	cargo +1.95.0 build --quiet --release --manifest-path apps/desktop-gpui/Cargo.toml --bin cap-gpui
	echo "✅ Build completed in ${SECONDS}s — running Cap GPUI"
	exec "$REPO_ROOT/apps/desktop-gpui/target/release/cap-gpui"
else
	echo "🔨 Building Cap GPUI..."
	cargo +1.95.0 build --quiet --manifest-path apps/desktop-gpui/Cargo.toml --bin cap-gpui
	echo "✅ Build completed in ${SECONDS}s — running Cap GPUI"
	exec "$REPO_ROOT/apps/desktop-gpui/target/debug/cap-gpui"
fi
