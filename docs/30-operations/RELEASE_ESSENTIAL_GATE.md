# Release Essential Gate

Authority:
- `cargo run -q -p bijux-dna-dev -- tooling run cargo-targets essential-release`
- `make gate-release-essential`
- `cargo run -q -p bijux-dna-dev -- tooling run certify-level1`

What this gate proves:
- essential architecture and domain integrity via `essential-integrity`
- canonical example smoke execution plus targeted dry-run and status contracts
- evidence verification and targeted planner/runtime evidence surfaces
- compatibility review across schema migration, route adapters, deprecation docs, and generated upgrade guidance via `essential-compatibility`
- refusal coverage and advisory/enforced admission coverage before a Level 1 release claim

Supporting truth:
- [docs/30-operations/BACKLOG_SCOREBOARD.md](BACKLOG_SCOREBOARD.md)

Smoke-only note:
- The paired `benchmark-smoke-level1` and `certify-level1` outputs measure deterministic bundle flow and artifact size only.
- They are not scientific-performance claims and must not be used as publication evidence.

## Scope
This document defines the operational or architecture surface for this workflow surface.

## Non-goals
- Replacing crate-level implementation details or test contracts.

## Contracts
- Changes to this surface must stay aligned with governed checks and generated references.

## Purpose
This document records the durable intent and enforcement boundary for this surface.
