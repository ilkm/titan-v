#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

limit=400
total=0

while IFS= read -r file; do
  lines="$(
    awk '
      /^\s*$/ { next }
      /^\s*\/\// { next }
      /^\s*\/\// { next }
      /^\s*\/\*/ { in_block=1; next }
      in_block && /\*\// { in_block=0; next }
      in_block { next }
      { count++ }
      END { print count+0 }
    ' "$file"
  )"
  if [ "$lines" -gt "$limit" ]; then
    total=$((total + 1))
    printf '%4d  %s\n' "$lines" "${file#"$repo_root"/}"
  fi
done < <(git ls-files '*.rs')

echo "total: $total" >&2
if [ "$total" -ne 0 ]; then
  exit 1
fi
