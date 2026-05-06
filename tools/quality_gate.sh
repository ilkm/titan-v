#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

echo "[1/6] cargo fmt"
cargo fmt

echo "[2/6] cargo check (-D warnings)"
RUSTFLAGS="-D warnings" cargo check --workspace

echo "[3/6] cargo clippy (-D warnings)"
cargo clippy --workspace -- -D warnings

echo "[4/6] cargo test"
cargo test --workspace

echo "[5/6] fn line limit check"
python3 tools/check_fn_code_lines.py

echo "[6/6] rs file line limit check"
./tools/check_rs_file_code_lines.sh
