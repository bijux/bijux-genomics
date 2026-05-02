# bijux-dna-runtime

## What this crate does
`bijux-dna-runtime` owns runtime contracts and recording helpers: run layout, manifests, records, provenance, observability contracts, telemetry events, and runner handoff types.

## What it must not do (boundaries)
It must not plan stages, choose tools, parse CLI/API requests, execute tools, invoke Docker/Apptainer, own analyzer/report behavior, or write outside declared run-layout roots.

## Role in the stack
Upstream callers provide execution plans, runner responses, runtime profiles, and declared run roots. Downstream consumers read schema-stable runtime artifacts for analysis, benchmarking, reproducibility, and reporting.

## Public API / entrypoints
Use root re-exports for stable runtime entrypoints and module namespaces for owner-specific details. See `docs/PUBLIC_API.md`, `docs/RUNTIME_CONTRACT.md`, and `docs/COMMANDS.md`.

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
Owns runtime artifact schemas, run-layout contracts, runner handoff contracts, telemetry contracts, provenance contracts, and canonical runtime writers.

## Artifacts / Contracts
See `docs/RUNTIME_CONTRACT.md`, `docs/ARTIFACTS.md`, and schema fixtures under `tests/fixtures/runtime_schema/`.

## Effects & determinism guarantees
Filesystem writes are limited to declared run-layout roots and runtime-owned artifact paths. Runtime does not spawn processes. See `docs/EFFECTS.md`.

## How to understand the crate in 10 minutes
- Read `tests/contracts/reference/reference_example.rs` for a concrete run story.
- Open `tests/fixtures/runtime_schema/default/run_manifest_v1.json` to see the canonical schema shape.

## How to run its tests
Run:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runtime --no-default-features
```

See `docs/TESTS.md`. Golden tests include `tests/contracts/reference/reference_example.rs`, `tests/schemas/schema/runtime_schema_snapshots.rs`, `tests/contracts/manifest_integrity.rs`, and `tests/contracts/run_layout_contract.rs`.

## Where the docs live
The crate root has only this `README.md`. All other crate docs live under `docs/`; start at `docs/INDEX.md`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes must update the relevant docs and tests in the same reviewable change.

## Repository Policy
This crate follows repository governance documentation. `/Users/bijan/bijux/bijux-g2/bijux-genomics/README.md`,
`README.md`, and `README.md`; re-read
those files before editing this child repository or making commits.
