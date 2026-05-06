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
set -e
set -u
set -o pipefail

script_source_path() {
	# bash: BASH_SOURCE[0], zsh: ${(%):-%N}, fallback: $0
	if [ -n "${BASH_SOURCE:-}" ]; then
		printf '%s\n' "${BASH_SOURCE[0]}"
		return
	fi
	if [ -n "${ZSH_VERSION:-}" ]; then
		printf '%s\n' "${(%):-%N}"
		return
	fi
	printf '%s\n' "$0"
}

ROOT="$(cd "$(dirname "$(script_source_path)")/.." && pwd)"
cd "$ROOT"

need_cmd() {
	local name="$1"
	local hint="$2"
	if ! command -v "$name" >/dev/null 2>&1; then
		echo "$name not found. $hint" >&2
		exit 1
	fi
}

has_cmd() {
	command -v "$1" >/dev/null 2>&1
}

need_cargo_packager() {
	need_cmd "cargo-packager" "Install with: cargo install cargo-packager --locked --version 0.11.8"
}

echo "==> cargo build --release (-p titan-center -p titan-host)"
cargo build --release -p titan-center -p titan-host

OS="$(uname -s || true)"
echo "==> detected platform: $OS"
case "$OS" in
Darwin)
	if has_cmd cargo-packager; then
		echo "==> macOS: .app + .dmg (cargo-packager)"
		cargo packager --release --packages titan-center,titan-host --formats dmg
		echo "Artifacts: $ROOT/target/release/*.dmg and *.app"
	else
		echo "==> cargo-packager not found; fallback to build-only."
		echo "    Install for DMG: cargo install cargo-packager --locked --version 0.11.8"
	fi
	;;
Linux)
	echo "==> Linux: release binaries only (no DMG/NSIS)."
	echo "    $ROOT/target/release/titan-center"
	echo "    $ROOT/target/release/titan-host"
	;;
MINGW* | MSYS* | CYGWIN*)
	if ! has_cmd cargo-packager; then
		echo "==> cargo-packager not found; fallback to build-only."
		echo "    Install for NSIS packaging: cargo install cargo-packager --locked --version 0.11.8"
	elif ! has_cmd makensis; then
		echo "==> makensis not found; fallback to build-only."
		echo "    Install NSIS and ensure 'makensis' is on PATH."
	else
		echo "==> Windows: NSIS installer .exe (cargo-packager + makensis)"
		cargo packager --release --packages titan-center,titan-host --formats nsis
		echo "Artifacts: $ROOT/target/release/*.exe (installer)"
	fi
	;;
*)
	echo "==> Unrecognized OS '$OS': leaving built binaries in target/release/ only." >&2
	;;
esac
