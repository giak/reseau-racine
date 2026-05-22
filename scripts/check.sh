#!/usr/bin/env bash
set -euo pipefail

echo "=== Checking RéseauRacine ==="

echo "→ cargo check"
cargo check --workspace

echo ""
echo "→ cargo clippy"
cargo clippy --workspace

echo ""
echo "→ cargo test"
cargo test --workspace

echo ""
echo "✓ All checks passed"
