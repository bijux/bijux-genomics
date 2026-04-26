# bijux-dna-analyze

## What this crate does
Loads recorded runtime artifacts, evaluates benchmark decisions, and renders durable analysis
artifacts.

## What it must not do (boundaries)
This crate analyzes completed runs. It does not plan workflows, execute tools, or own runtime
layout policy.

## Effects & determinism guarantees
The crate computes deterministic summaries and report outputs from recorded runtime artifacts.
It does not mutate runtime inputs and keeps report shape/versioning under documented contracts.

## Public API / entrypoints
Start with `docs/PUBLIC_API.md` for the curated surface and `docs/ARCHITECTURE.md` for the internal
tree. The crate root keeps a single execution entrypoint in `src/lib.rs`, while stable exports are
curated in `src/public_api/mod.rs`.

## Key contracts it owns/consumes
- report structure: `docs/REPORT_CONTRACT.md`
- decision semantics: `docs/DECISIONS.md`
- schema compatibility: `docs/SCHEMA.md`
- determinism: `docs/DETERMINISM.md`
- change policy: `docs/CHANGE_RULES.md`

## Artifacts / Contracts
Inputs:
- typed `AnalyzeInput` requests from `src/api/`
- runtime facts, summaries, manifests, and SQLite-backed records loaded through `src/load/`

Outputs:
- `AnalyzeOutput` responses returned by `analyze_run`
- durable report artifacts rendered through `src/report/`
- dashboard facts and run summaries emitted from `src/exports/`

## Failure modes
- contract/schema mismatch in loaded runtime artifacts
- missing or corrupted run summaries/manifests
- invalid benchmark decision state for requested report projections

## Internal layout
- `src/contracts/`: versioned analysis contract handshake
- `src/diagnostics/`: durable error taxonomies for load and aggregate flows
- `src/exports/`: run summary, stage summary, and dashboard facts writers
- `src/pipeline/steps/`: canonical load, validate, compute, report, and render pipeline steps
- `src/report/`: report construction, rendering, and section assembly

## How to run its tests
See `docs/TESTS.md` for the full map. The test tree is split by enduring intent:
- `tests/boundaries.rs`: architecture and public-surface guardrails
- `tests/contracts.rs`: report, facts, pipeline, and public contract behavior
- `tests/determinism.rs`: fixture stability guarantees
- `tests/schemas.rs`: SQLite schema and migration coverage
- `tests/semantics.rs`: ranking and decision semantics

## Where the docs live
Primary docs live in `docs/INDEX.md`. Test guidance is in `docs/TESTS.md`.
