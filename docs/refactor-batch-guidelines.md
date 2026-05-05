# Refactor Batch Guidelines

This document defines the default rules for structural refactors in this repository.

## Scope

- Keep behavior unchanged in each batch.
- Prefer code movement and module extraction over semantic rewrites.
- Split by responsibility first, then by naming cleanup.

## Required Checks Per Batch

Run from repo root in this order:

1. `cargo fmt`
2. `RUSTFLAGS='-D warnings' cargo check --workspace`
3. `cargo clippy --workspace -- -D warnings`
4. `cargo test --workspace`
5. `python3 tools/check_fn_code_lines.py`

## Module Boundary Rules

- Entry files should orchestrate and dispatch only.
- Domain logic belongs to focused submodules.
- Shared UI math/layout helpers should live in `titan-egui-widgets`.
- Keep duplicate business forks explicit; only extract pure shared helpers.

## Migration Discipline

- One batch should target one area (`net_inbox`, `serve/run`, `device_store`, etc.).
- Avoid mixing transport/protocol changes with structure refactors.
- Preserve existing public API paths unless migration is explicitly planned.
