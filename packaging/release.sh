#!/usr/bin/env bash
# Build Titan Center + Titan Host for distribution.
#
# Outputs (under workspace target/release/):
#   macOS:   Titan Center.app, Titan Host.app, and *_aarch64.dmg (or *_x64.dmg) via cargo-packager.
#   Windows: NSIS setup .exe installers (requires NSIS: https://nsis.sourceforge.io/ — `makensis` on PATH).
#   Linux:   Plain release binaries `titan-center` and `titan-host` only (no installer).
#
# Prerequisite (macOS / Windows packaging):
#   cargo install cargo-packager --locked --version 0.11.8
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

need_cargo_packager() {
	if ! command -v cargo-packager >/dev/null 2>&1; then
		echo "cargo-packager not found. Install with:" >&2
		echo "  cargo install cargo-packager --locked --version 0.11.8" >&2
		exit 1
	fi
}

echo "==> cargo build --release (-p titan-center -p titan-host)"
cargo build --release -p titan-center -p titan-host

OS="$(uname -s || true)"
case "$OS" in
Darwin)
	need_cargo_packager
	echo "==> macOS: .app + .dmg (cargo-packager)"
	cargo packager --release --packages titan-center,titan-host --formats dmg
	echo "Artifacts: $ROOT/target/release/*.dmg and *.app"
	;;
Linux)
	echo "==> Linux: release binaries only (no DMG/NSIS)."
	echo "    $ROOT/target/release/titan-center"
	echo "    $ROOT/target/release/titan-host"
	;;
MINGW* | MSYS* | CYGWIN*)
	need_cargo_packager
	echo "==> Windows: NSIS installer .exe (cargo-packager; needs makensis on PATH)"
	cargo packager --release --packages titan-center,titan-host --formats nsis
	echo "Artifacts: $ROOT/target/release/*.exe (installer)"
	;;
*)
	echo "==> Unrecognized OS '$OS': leaving built binaries in target/release/ only." >&2
	;;
esac
