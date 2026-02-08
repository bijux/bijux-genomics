# Architecture

## Contract authority
See `CONTRACT_MAP.md` for the authoritative map of contracts and where they live.

## Modules (by area)
- `contract/*` (execution, run, tooling, version)
- `ids.rs`
- `foundation/*`

## Data flow
- Contracts are defined here and consumed by every other crate.
