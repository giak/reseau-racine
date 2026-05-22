#!/usr/bin/env bash
set -euo pipefail

echo "=== Checking RéseauRacine ==="

scripts_dir="$(dirname "$0")"

echo "→ cargo check"
$scripts_dir/dev.sh cargo check --workspace --exclude rr-tauri

echo ""
echo "→ cargo clippy"
$scripts_dir/dev.sh cargo clippy --workspace --exclude rr-tauri

echo ""
echo "→ cargo test"
$scripts_dir/dev.sh cargo test --workspace --exclude rr-tauri

echo ""
echo "✓ All checks passed"
