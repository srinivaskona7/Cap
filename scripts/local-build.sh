#!/usr/bin/env bash
# Quick local build script for sricap-rec with caching enabled
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "==> Building sricap-rec (using local incremental cache)..."
cd "$REPO_ROOT/apps/desktop-gpui"

# Check if incremental cache is active
export CARGO_INCREMENTAL=1

MODE="${1:-check}"

case "$MODE" in
  check)
    echo "==> Running fast incremental type-check..."
    cargo check --bin sricap-rec
    echo "==> Done in seconds! [Check Passed]"
    ;;
  build)
    echo "==> Building local executable binary without launching..."
    cargo build --bin sricap-rec
    echo "==> Binary ready at: apps/desktop-gpui/target/debug/sricap-rec"
    ;;
  release)
    echo "==> Building optimized release binary..."
    cargo build --release --bin sricap-rec
    echo "==> Release binary ready at: apps/desktop-gpui/target/release/sricap-rec"
    ;;
  bundle)
    echo "==> Building standalone DMG & .app installers..."
    cd "$REPO_ROOT"
    ./scripts/build-desktop-binaries.sh aarch64-apple-darwin
    CI=false node scripts/bundle-gpui.mjs aarch64-apple-darwin --bundles app,dmg
    echo "==> Standalone DMG ready in: apps/desktop-gpui/target/aarch64-apple-darwin/release/bundle/dmg/"
    ;;
  stats)
    echo "==> Local Cache Statistics:"
    if [ -d "$REPO_ROOT/apps/desktop-gpui/target/debug/incremental" ]; then
      du -sh "$REPO_ROOT/apps/desktop-gpui/target/debug/incremental" 2>/dev/null || true
    else
      echo "No incremental cache yet. Run './scripts/local-build.sh' once to populate."
    fi
    ;;
  *)
    echo "Usage: $0 [check|build|release|bundle|stats]"
    echo "  check    - (Default) Fastest ~2s type check"
    echo "  build    - Incremental debug binary build"
    echo "  release  - Fast release binary build"
    echo "  bundle   - Full macOS .app & .dmg packaging"
    echo "  stats    - View current cache size"
    exit 1
    ;;
esac
