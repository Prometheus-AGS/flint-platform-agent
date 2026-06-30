#!/usr/bin/env bash
# Flint Platform Agent — canonical CI check.
# Runs locally and unchanged in CI. Mirrors the sibling planes' gate.
set -euo pipefail

echo "==> rustfmt --check"
cargo fmt --all --check

echo "==> clippy (pedantic, -D warnings)"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> cargo check"
cargo check --workspace

echo "==> cargo test"
cargo test --workspace

echo "OK: fmt + clippy::pedantic + cargo check + tests all green"
