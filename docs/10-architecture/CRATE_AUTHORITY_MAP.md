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
- `bijux-dna-planner-fastq` and `bijux-dna-planner-bam`: planner authority for stage selection and plan assembly.
- `bijux-dna-stages-fastq` and `bijux-dna-stages-bam`: stage authority for invocation/parsing contracts.
- `bijux-dna-api`: stable API surface orchestrating planner/engine/environment contracts.
- `bijux-dna`: CLI adapters over API/registry/domain commands.
- `bijux-dna-analyze`: report/analytics/provenance contracts over produced artifacts.

## Non-goals
- Duplicating low-level dependency edge tables (see [docs/10-architecture/BOUNDARY_MAP.md](BOUNDARY_MAP.md)).

## Contracts
- Ownership source of truth: this document.
- Boundary map and allowed edges: [docs/10-architecture/BOUNDARY_MAP.md](BOUNDARY_MAP.md)
- Workspace policy diagnostics: [crates/bijux-dna-policies/docs/ENFORCEMENT.md](../../crates/bijux-dna-policies/docs/ENFORCEMENT.md)

## Enforcement
- Dependency edges: [crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs](../../crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs)
- Dependency graph: [crates/bijux-dna-policies/tests/boundaries/deps/graph/dependency_graph.rs](../../crates/bijux-dna-policies/tests/boundaries/deps/graph/dependency_graph.rs)
- Effect boundaries: [crates/bijux-dna-policies/tests/boundaries/deps/graph/effect_boundary_map.rs](../../crates/bijux-dna-policies/tests/boundaries/deps/graph/effect_boundary_map.rs)
- Command spawn confinement:
  [crates/bijux-dna-policies/tests/contracts/tooling/governance_core/command_spawn_policy.rs](../../crates/bijux-dna-policies/tests/contracts/tooling/governance_core/command_spawn_policy.rs)
- Domain/stage purity and crate responsibilities:
  [crates/bijux-dna-policies/tests/contracts/tooling/governance/purity_effects_responsibility_policy.rs](../../crates/bijux-dna-policies/tests/contracts/tooling/governance/purity_effects_responsibility_policy.rs)

## Examples
- Planner-only tool selection logic lives in `crates/bijux-dna-planner-*`; execution wiring lives in `crates/bijux-dna-engine`.
- Stage contracts (`crates/bijux-dna-stages-*`) define invocation and parsing boundaries, while domain truth (`crates/bijux-dna-domain-*`) owns IDs and invariants.

## Failure modes
- Ownership drift: crates begin to absorb responsibilities owned elsewhere, creating duplicated logic and policy violations.
- Boundary erosion: planner/execution/runtime concerns blend, reducing determinism and making failures harder to localize.
