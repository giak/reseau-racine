#!/usr/bin/env bash
set -euo pipefail

echo "=== Building RéseauRacine ==="

cargo build --workspace "$@"

echo "✓ Build complete"
