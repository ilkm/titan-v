#!/usr/bin/env python3
"""List Rust functions whose body *code lines* exceed a limit.

Function counting scope:
- Count only code lines inside the outermost function `{ ... }` body.
- Do NOT count signature/parameter lines.
- Do NOT count blank lines, pure comment lines, or lines that are only `{` / `}`.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

LIMIT = 30
LINE_RECOMMENDED = 80
LINE_LIMIT = 120
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


def is_counted_body_code_line(line: str) -> bool:
    s = line.strip()
    if not is_code_line(line):
        return False
    if s in {"{", "}"}:
        return False
    return True


FN_RE = re.compile(
    r"^\s*(?:#\[[^\]]*\]\s*)*(?:(?:pub|pub\(crate\)|pub\(super\)|pub\(self\))\s+)*"
    r"(?:async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\b"
)


def scan_rust_file(path: Path) -> list[tuple[int, int, str]]:
    """Return list of (start_line_1based, body_code_lines, name) for functions over LIMIT."""
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)
    body_code_flags = [is_counted_body_code_line(l) for l in lines]
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
        bounds = find_fn_body_bounds(tail)
        if bounds is None:
            i += 1
            continue
        body_open_char, body_close_char = bounds
        body_open_line = char_to_line(lines, i, body_open_char)
        body_close_line = char_to_line(lines, i, body_close_char)
        if body_open_line is None or body_close_line is None:
            i += 1
            continue
        body_code_lines = sum(
            1
            for k in range(body_open_line + 1, body_close_line)
            if body_code_flags[k]
        )
        if body_code_lines > LIMIT:
            out.append((start + 1, body_code_lines, name))
        i = body_close_line + 1
    return out


def char_to_line(lines: list[str], line_offset: int, char_pos: int) -> int | None:
    consumed = 0
    for j in range(line_offset, len(lines)):
        chunk = lines[j]
        if consumed + len(chunk) > char_pos:
            return j
        consumed += len(chunk)
    if lines:
        return len(lines) - 1
    return None


def scan_long_lines(path: Path) -> list[tuple[int, int]]:
    """Return list of (line_1based, line_length) for code lines over LINE_LIMIT."""
    text = path.read_text(encoding="utf-8")
    out: list[tuple[int, int]] = []
    for idx, line in enumerate(text.splitlines(), start=1):
        if not is_code_line(line):
            continue
        line_length = len(line)
        if line_length > LINE_LIMIT:
            out.append((idx, line_length))
    return out


def find_fn_body_bounds(src: str) -> tuple[int, int] | None:
    """Return (open_brace_idx, close_brace_idx) for function body, or None."""
    # Find first `{` that starts the body (after fn ...). Skip generics/parens crudely.
    state = "seek_brace"  # before outer {
    brace = 0
    body_open = -1
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
                body_open = i
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
                return (body_open, i)
        i += 1
    return None


def _char_literal_start(src: str, i: int) -> bool:
    """Heuristic: `'` at position i starts a char literal (vs a lifetime label).

    A lifetime is `'IDENT` where IDENT is `[a-zA-Z_][a-zA-Z0-9_]*` and is never
    followed by a closing `'`. A char literal always ends with `'` (e.g. `'a'`,
    `'\\n'`, `'b'`). When the next char is `_` or alpha, we walk the identifier
    and only treat it as a char literal when a closing `'` immediately follows.
    """
    if i + 1 >= len(src):
        return False
    nxt = src[i + 1]
    if nxt == "\\":
        return True
    if nxt == "'":
        return False
    if nxt == "_" or nxt.isalpha():
        j = i + 1
        while j < len(src) and (src[j].isalnum() or src[j] == "_"):
            j += 1
        return j < len(src) and src[j] == "'"
    return True


def main() -> None:
    over: list[tuple[Path, int, int, str]] = []
    long_lines: list[tuple[Path, int, int]] = []
    for path in sorted(ROOT.rglob("*.rs")):
        if "target" in path.parts or ".git" in path.parts:
            continue
        rel = path.relative_to(ROOT)
        for start_line, code_lines, name in scan_rust_file(path):
            over.append((rel, start_line, code_lines, name))
        for line_no, line_length in scan_long_lines(path):
            long_lines.append((rel, line_no, line_length))

    over.sort(key=lambda t: (-t[2], str(t[0])))
    for p, ln, cl, name in over:
        print(f"{cl:4d}  {p}:{ln}  fn {name}")
    long_lines.sort(key=lambda t: (-t[2], str(t[0]), t[1]))
    for p, ln, ll in long_lines:
        print(
            f"{ll:4d}  {p}:{ln}  line-length>{LINE_LIMIT} (recommended<={LINE_RECOMMENDED})"
        )
    print(f"total: {len(over)}", file=sys.stderr)
    print(f"line_total: {len(long_lines)}", file=sys.stderr)
    sys.exit(1 if over or long_lines else 0)


if __name__ == "__main__":
    main()
