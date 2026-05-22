#!/usr/bin/env bash
set -euo pipefail

echo "=== Release build ==="

scripts_dir="$(dirname "$0")"

$scripts_dir/dev.sh cargo build --release --package rr-cli

echo ""
echo "Binary: target/release/rr"
$scripts_dir/dev.sh sh -c "du -h target/release/rr | cut -f1"
