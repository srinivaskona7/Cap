#!/usr/bin/env bash
# stop.sh - Terminate all sricap-rec, Cap, dev servers, and cargo build processes
set -euo pipefail

QUIET=false
if [[ "${1:-}" == "--quiet" ]] || [[ "${1:-}" == "-q" ]]; then
	QUIET=true
fi

if [ "$QUIET" = false ]; then
	echo "🛑 Stopping all sricap-rec and Cap processes..."
fi

# 1. Kill native application instances
pkill -f "sricap-rec" 2>/dev/null || true
pkill -f "Cap.app" 2>/dev/null || true
pkill -f "cap-desktop-gpui" 2>/dev/null || true
pkill -f "cap-muxer" 2>/dev/null || true
pkill -f "cap-cli" 2>/dev/null || true

# 2. Kill background cargo builds or checkers running for desktop-gpui
pkill -f "cargo.*desktop-gpui" 2>/dev/null || true
pkill -f "cargo.*sricap-rec" 2>/dev/null || true

# 3. Kill web dev servers if active
pkill -f "bun.*dev:desktop" 2>/dev/null || true
pkill -f "tauri dev" 2>/dev/null || true

if [ "$QUIET" = false ]; then
	echo "✅ All processes stopped successfully."
fi
