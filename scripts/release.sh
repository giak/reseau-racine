#!/usr/bin/env bash
set -euo pipefail

echo "=== Release build ==="

cargo build --release

echo ""
echo "Binary: target/release/rr"
echo "Size: $(du -h target/release/rr | cut -f1)"
