#!/usr/bin/env bash
set -euo pipefail

echo "=== Building RéseauRacine ==="

scripts_dir="$(dirname "$0")"

$scripts_dir/dev.sh cargo build --workspace --exclude rr-tauri "$@"

echo "✓ Build complete"
