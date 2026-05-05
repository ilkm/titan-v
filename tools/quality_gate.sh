#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

echo "[1/5] cargo fmt"
cargo fmt

echo "[2/5] cargo check (-D warnings)"
RUSTFLAGS="-D warnings" cargo check --workspace

echo "[3/5] cargo clippy (-D warnings)"
cargo clippy --workspace -- -D warnings

echo "[4/5] cargo test"
cargo test --workspace

echo "[5/5] fn line limit check"
python3 tools/check_fn_code_lines.py
