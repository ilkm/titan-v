#!/usr/bin/env python3
"""List Rust functions whose *code lines* (general.mdc口径) exceed a limit.

Span: from the line containing `fn` through the line containing the matching
closing `}` of the function body (signature + body). Excludes lines that are
only blank, only // or /// or //!, or only block-comment.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

LIMIT = 30
ROOT = Path(__file__).resolve().parents[1]


def is_code_line(line: str) -> bool:
    s = line.strip()
    if not s:
        return False
    if s.startswith("///") or s.startswith("//!"):
        return False
    if s.startswith("//"):
        return False
    if re.match(r"^/\*", s):
        return False
    return True


FN_RE = re.compile(
    r"^\s*(?:#\[[^\]]*\]\s*)*(?:(?:pub|pub\(crate\)|pub\(super\)|pub\(self\))\s+)*"
    r"(?:async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\b"
)


def scan_rust_file(path: Path) -> list[tuple[int, int, str]]:
    """Return list of (start_line_1based, code_lines, name) for functions over LIMIT."""
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)
    code_flags = [is_code_line(l) for l in lines]
    out: list[tuple[int, int, str]] = []

    i = 0
    while i < len(lines):
        line = lines[i]
        m = FN_RE.match(line)
        if not m:
            i += 1
            continue
        name = m.group(1)
        # Skip `fn` inside macro invocations heuristically: must not be `macro_rules!`
        if i > 0 and "macro_rules!" in lines[i - 1]:
            i += 1
            continue

        start = i
        # Join from fn line for brace scan
        tail = "".join(lines[i:])
        end_rel = find_fn_body_end(tail)
        if end_rel is None:
            i += 1
            continue
        end_char = end_rel
        # Map char offset to end line
        consumed = 0
        end_line = start
        for j in range(i, len(lines)):
            chunk = lines[j]
            if consumed + len(chunk) > end_char:
                end_line = j
                break
            consumed += len(chunk)
        else:
            end_line = len(lines) - 1

        code_lines = sum(1 for k in range(start, end_line + 1) if code_flags[k])
        if code_lines > LIMIT:
            out.append((start + 1, code_lines, name))
        i = end_line + 1
    return out


def find_fn_body_end(src: str) -> int | None:
    """Return index after last char of matching top-level `}` for fn body, or None."""
    # Find first `{` that starts the body (after fn ...). Skip generics/parens crudely.
    state = "seek_brace"  # before outer {
    brace = 0
    i = 0
    n = len(src)
    # Lexer states
    NORMAL, LINE_COMMENT, BLOCK_COMMENT, STRING, CHAR, RAW_STRING = range(6)
    lex = NORMAL
    raw_hashes = 0

    while i < n:
        c = src[i]
        nxt = src[i + 1] if i + 1 < n else ""

        if lex == LINE_COMMENT:
            if c == "\n":
                lex = NORMAL
            i += 1
            continue
        if lex == BLOCK_COMMENT:
            if c == "*" and nxt == "/":
                lex = NORMAL
                i += 2
                continue
            i += 1
            continue
        if lex == STRING:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                lex = NORMAL
            i += 1
            continue
        if lex == CHAR:
            if c == "\\":
                i += 2
                continue
            if c == "'":
                lex = NORMAL
            i += 1
            continue
        if lex == RAW_STRING:
            if c == '"' and nxt == "#" * raw_hashes:
                # end raw string
                i += 1 + raw_hashes
                lex = NORMAL
                continue
            i += 1
            continue

        # NORMAL
        if c == "/" and nxt == "/":
            lex = LINE_COMMENT
            i += 2
            continue
        if c == "/" and nxt == "*":
            lex = BLOCK_COMMENT
            i += 2
            continue
        if c == "b" and nxt == '"':
            lex = STRING
            i += 2
            continue
        if c == '"':
            lex = STRING
            i += 1
            continue
        if c == "'" and _char_literal_start(src, i):
            lex = CHAR
            i += 1
            continue
        if c in "rR" and nxt == "#":
            # r#" or br#"
            j = i + 1
            h = 0
            while j < n and src[j] == "#":
                h += 1
                j += 1
            if j < n and src[j] == '"':
                lex = RAW_STRING
                raw_hashes = h
                i = j + 1
                continue
        if c == "r" and nxt == '"':
            lex = RAW_STRING
            raw_hashes = 0
            i += 2
            continue

        if state == "seek_brace":
            if c == "{":
                state = "in_body"
                brace = 1
                i += 1
                continue
            i += 1
            continue

        # in_body
        if c == "{":
            brace += 1
        elif c == "}":
            brace -= 1
            if brace == 0:
                return i + 1
        i += 1
    return None


def _char_literal_start(src: str, i: int) -> bool:
    """Heuristic: `'` starts char if next is escape or single non-quote char then '."""
    if i + 1 >= len(src):
        return False
    nxt = src[i + 1]
    if nxt == "\\":
        return True
    if nxt == "'":
        return False
    # lifetime 'a
    if nxt.isalpha() and i + 2 < len(src) and src[i + 2].isalnum():
        return False
    return True


def main() -> None:
    over: list[tuple[Path, int, int, str]] = []
    for path in sorted(ROOT.rglob("*.rs")):
        if "target" in path.parts or ".git" in path.parts:
            continue
        for start_line, code_lines, name in scan_rust_file(path):
            over.append((path.relative_to(ROOT), start_line, code_lines, name))

    over.sort(key=lambda t: (-t[2], str(t[0])))
    for p, ln, cl, name in over:
        print(f"{cl:4d}  {p}:{ln}  fn {name}")
    print(f"total: {len(over)}", file=sys.stderr)
    sys.exit(1 if over else 0)


if __name__ == "__main__":
    main()
