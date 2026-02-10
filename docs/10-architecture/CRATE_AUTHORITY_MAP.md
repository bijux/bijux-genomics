# Crate Authority Map

## What
Defines authoritative ownership boundaries for planning, execution, runtime resolution, and domain truth.

## Why
Prevents responsibility drift across crates and makes policy failures actionable.

## Authority
- `bijux-dna-domain-*`: authored scientific/domain truth only (IDs, vocabularies, constraints, typed models).
- `bijux-dna-domain-compiler`: compiles domain SSOT into generated config views.
- `bijux-dna-stages-*`: pure stage contracts/invocation builders/parsers only.
- `bijux-dna-planner-*`: plan assembly + deterministic selection/explanation only.
- `bijux-dna-pipelines`: default pipeline composition + defaults ledger ownership.
- `bijux-dna-engine`: executes explicit plans only; no planning or backend execution details.
- `bijux-dna-runner`: backend process/container execution only (Docker/Apptainer invocation boundary).
- `bijux-dna-environment`: image/runtime resolution and environment probing only.
- `bijux-dna-api`: stable API surface orchestrating planner/engine/environment contracts.
- `bijux-dna-cli`: CLI adapters over API/registry/domain commands.
- `bijux-dna-analyze`: report/analytics/provenance contracts over produced artifacts.

## Non-goals
- Duplicating low-level dependency edge tables (see `docs/10-architecture/BOUNDARY_MAP.md`).

## Enforcement
- Dependency edges: `crates/bijux-dna-policies/tests/boundaries/deps/*.rs`
- Effect boundaries: `crates/bijux-dna-policies/tests/boundaries/deps/effect_boundary_map.rs`
- Command spawn confinement: `crates/bijux-dna-policies/tests/contracts/tooling/command_spawn_policy.rs`
- Domain/stage purity and crate responsibilities:
  `crates/bijux-dna-policies/tests/contracts/tooling/purity_effects_responsibility_policy.rs`
