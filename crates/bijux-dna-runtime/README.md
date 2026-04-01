# bijux-dna-runtime

## What this crate does
Runtime contracts + recording: run layout, manifests, records, provenance, and telemetry events.

## What it must not do (boundaries)
No tool execution; only writes under run layout.

## Role in the stack
Upstream: engine/runner. Downstream: analyze/benchmark.

## Public API / entrypoints
See `crates/bijux-dna-runtime/docs/INDEX.md`, `crates/bijux-dna-runtime/docs/RUNTIME_CONTRACT.md`, `crates/bijux-dna-runtime/docs/ARTIFACTS.md`, `crates/bijux-dna-runtime/docs/OBSERVABILITY.md`, `crates/bijux-dna-runtime/docs/EVENTS.md`, `crates/bijux-dna-runtime/docs/BOUNDARY.md`, `crates/bijux-dna-runtime/docs/GLOSSARY.md`, `crates/bijux-dna-runtime/docs/CHANGE_RULES.md`.

## Truth artifacts (canonical runtime outputs)
Schema-stable artifacts owned by runtime:
- Run layout contract: `tests/fixtures/runtime_schema/default/run_layout_v1.json`
- Run manifest contract: `tests/fixtures/runtime_schema/default/run_manifest_v1.json`
- Run record contract: `tests/fixtures/runtime_schema/default/run_record_v1.json`
- Run provenance contract: `tests/fixtures/runtime_schema/default/run_provenance_v1.json`

## Exact JSON artifacts owned (stability expectations)
Stable schemas (strict compatibility, versioned on change):
- `RunLayoutV1`
- `RunManifest`
- `RunRecordV1`
- `RunProvenanceV1`

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
See `crates/bijux-dna-runtime/docs/RUNTIME_CONTRACT.md`, `crates/bijux-dna-runtime/docs/ARTIFACTS.md`, and schema fixtures under `tests/fixtures/runtime_schema/`.

## Effects & determinism guarantees
Filesystem writes under run layout only. See `crates/bijux-dna-runtime/docs/EFFECTS.md` and the golden tests below.

## How to understand the crate in 10 minutes
- Read `tests/contracts/reference/reference_example.rs` for a concrete run story.
- Open `tests/fixtures/runtime_schema/default/run_manifest_v1.json` to see the canonical schema shape.

## How to run its tests
See `crates/bijux-dna-runtime/docs/TESTS.md`. Golden tests: `tests/contracts/reference/reference_example.rs`, `tests/schemas/schema/runtime_schema_snapshots.rs`, `tests/contracts/manifest_integrity.rs`, `tests/contracts/run_layout_contract.rs`.

## Where the docs live
Start at `crates/bijux-dna-runtime/docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-runtime/docs/CHANGE_RULES.md`.
