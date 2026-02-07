# Boundary Map

Owner: Architecture
Scope: Contract and boundary authority
Last reviewed: 2026-02-07
Contract version: v1
Applies to crates: bijux-core, bijux-engine, bijux-runtime, bijux-runner, bijux-api

## What
Points to the canonical boundary diagram and dependency rules.

## Why
Avoids boundary duplication across documents.

## Non-goals
- Restating dependency rules.

## Contracts
Enforced by:
- `docs/10-architecture/BOUNDARY_DIAGRAM.md`
- `docs/10-architecture/DEPENDENCY_RULES.md`
- `crates/bijux-policies/tests/deps/dependency_boundaries.rs`
- `crates/bijux-policies/tests/deps/effect_boundary_map.rs`

## Examples
See `BOUNDARY_DIAGRAM.md` for the canonical diagram.

## Failure modes
Boundary violations fail CI dependency/effect policies.
