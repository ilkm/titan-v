#!/usr/bin/env bash
# Fail if any workspace .rs source (excluding target/) exceeds MAX lines.
set -euo pipefail
MAX="${1:-400}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bad=0
while IFS= read -r -d '' f; do
  n=$(wc -l <"$f" | tr -d ' ')
  if [[ "$n" -gt "$MAX" ]]; then
    echo "over ${MAX} lines ($n): $f" >&2
    bad=1
  fi
done < <(find "$ROOT/crates" -name '*.rs' -not -path '*/target/*' -print0)
exit "$bad"
