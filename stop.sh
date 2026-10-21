#!/usr/bin/env bash

if [[ "$1" != "--quiet" ]]; then
	echo "🛑 Stopping all Cap processes..."
fi

pkill -f "Cap.app" 2>/dev/null || true
pkill -f "cap-gpui" 2>/dev/null || true
pkill -f "cargo.*desktop-gpui" 2>/dev/null || true

if [[ "$1" != "--quiet" ]]; then
	echo "✅ All Cap processes stopped."
fi
