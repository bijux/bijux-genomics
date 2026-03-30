# Architecture

This document stays brief by design. Use it as a map, then follow the linked docs for the full contract.

## Layout
- `lib.rs` exposes the stable library surface.
- `main.rs` stays binary-only and delegates into `cli/`.
- `model/` owns ENA query, record, and manifest types.
- `client/` owns request building, parsing, and client-facing errors.
- `download/` owns planning, transfer execution, and result reporting.
- `cli/` owns argument parsing, command dispatch, and manifest output.

## Change rules
- Add new root files only for enduring crate-level concerns.
- Prefer focused submodules over growing `main.rs`, `client/mod.rs`, or `download/mod.rs`.
- Update this map and the boundary tree contract together when the layout changes intentionally.

## Pointers
- `INDEX.md` for the documentation map.
- `SCOPE.md` for crate boundaries.
- `TESTS.md` for verification coverage.
