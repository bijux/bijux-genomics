# bijux-dna-runtime

## What this crate does
Runtime contracts + recording: run layout, manifests, records, provenance, and telemetry events.

## What it must not do (boundaries)
No tool execution; only writes under run layout.

## Role in the stack
Upstream: engine/runner. Downstream: analyze/benchmark.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/RUNTIME_CONTRACT.md`, `docs/ARTIFACTS.md`, `docs/OBSERVABILITY.md`, `docs/EVENTS.md`, `docs/BOUNDARY.md`, `docs/GLOSSARY.md`, `docs/CHANGE_RULES.md`.

## Truth artifacts (canonical runtime outputs)
Schema-stable artifacts owned by runtime:
- Run layout: `run_layout.json` (schema: `tests/fixtures/runtime_schema/default/run_layout_v1.json`)
- Run manifest: `run_manifest.json` (schema: `tests/fixtures/runtime_schema/default/run_manifest_v1.json`)
- Run record: `run_record.json` (schema: `tests/fixtures/runtime_schema/default/run_record_v1.json`)
- Run provenance: `run_provenance.json` (schema: `tests/fixtures/runtime_schema/default/run_provenance_v1.json`)

## Exact JSON artifacts owned (stability expectations)
Stable schemas (strict compatibility, versioned on change):
- `run_layout.json`
- `run_manifest.json`
- `run_record.json`
- `run_provenance.json`

Stable per-step artifacts (strict compatibility, emitted under run layout):
- `effective_config.json`
- `tool_invocation.json`
- `metrics.json`
- `stage_report.json`
- `execution_record.json`

Telemetry outputs (schema-stable fields; timestamps are unstable by design):
- `events.jsonl`
- `timings.json`
- `resources.json`
- `errors.json`

## Key contracts it owns/consumes
Owns runtime artifact schemas and run layout contract.

## Artifacts / Contracts
See `docs/RUNTIME_CONTRACT.md`, `docs/ARTIFACTS.md`, and schema fixtures under `tests/fixtures/runtime_schema/`.

## Effects & determinism guarantees
Filesystem writes under run layout only. See `docs/EFFECTS.md` and the golden tests below.

## How to understand the crate in 10 minutes
- Read `tests/reference/reference_example.rs` for a concrete run story.
- Open `tests/fixtures/runtime_schema/default/run_manifest_v1.json` to see the canonical schema shape.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/reference/reference_example.rs`, `tests/schema/runtime_schema_snapshots.rs`, `tests/contracts/manifest_integrity.rs`, `tests/contracts/run_layout_contract.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
