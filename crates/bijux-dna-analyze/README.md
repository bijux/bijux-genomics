# bijux-dna-analyze

## What this crate does
Loads recorded runtime artifacts, evaluates benchmark decisions, and renders durable analysis
artifacts.

## Boundaries
This crate analyzes completed runs. It does not plan workflows, execute tools, or own runtime
layout policy.

## Public entrypoints
Start with `PUBLIC_API.md` for the curated surface and `docs/ARCHITECTURE.md` for the internal
tree. The crate root keeps a single execution entrypoint in `src/lib.rs`, while stable exports are
curated in `src/public_api/mod.rs`.

## Inputs and outputs
Inputs:
- typed `AnalyzeInput` requests from `src/api/`
- runtime facts, summaries, manifests, and SQLite-backed records loaded through `src/load/`

Outputs:
- `AnalyzeOutput` responses returned by `analyze_run`
- durable report artifacts rendered through `src/report/`
- dashboard facts and run summaries emitted from `src/exports/`

## Internal layout
- `src/contracts/`: versioned analysis contract handshake
- `src/diagnostics/`: durable error taxonomies for load and aggregate flows
- `src/exports/`: run summary, stage summary, and dashboard facts writers
- `src/pipeline/steps/`: canonical load, validate, compute, report, and render pipeline steps
- `src/report/`: report construction, rendering, and section assembly

## Contracts and operating rules
- report structure: `docs/REPORT_CONTRACT.md`
- decision semantics: `docs/DECISIONS.md`
- schema compatibility: `docs/SCHEMA.md`
- determinism: `docs/DETERMINISM.md`
- change policy: `docs/CHANGE_RULES.md`

## Tests
See `docs/TESTS.md` for the full map. The test tree is split by enduring intent:
- `tests/boundaries.rs`: architecture and public-surface guardrails
- `tests/contracts.rs`: report, facts, pipeline, and public contract behavior
- `tests/determinism.rs`: fixture stability guarantees
- `tests/schemas.rs`: SQLite schema and migration coverage
- `tests/semantics.rs`: ranking and decision semantics
