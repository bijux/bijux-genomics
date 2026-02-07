# Domain Template Checklist

Owner: Architecture
Scope: Domain crate structure and contracts.
Last reviewed: 2026-02-07
Contract version: v1

## Required Layout
- `src/metrics`
- `src/params`
- `src/types`
- `src/invariants`
- `src/stage_specs` (or `src/stage_specs.rs`)
- `src/pipeline_contract.rs`

## Required Files
- `src/lib.rs`
- `src/prelude.rs`

## Notes
Use this checklist when adding new domain crates or refactoring domain layouts.
