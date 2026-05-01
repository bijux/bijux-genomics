# bijux-dna-core

`bijux-dna-core` owns the shared contract, identifier, metric, canonicalization,
hashing, and foundation types used across `bijux-genomics`.

This crate follows repository governance documentation. `/Users/bijan/bijux/bijux-genomics/README.md`,
`README.md`, and `README.md`; re-read
those files before editing this child repository and before committing.

## Scope

This crate owns:

- Canonical contract types for execution graphs, run records, run metadata,
  provenance, tooling, and selection.
- Canonical JSON and deterministic hashing helpers.
- Typed identifiers and identifier parsing rules.
- Canonical pipeline, stage, and tool identifier catalogs.
- Shared metric identifiers, schemas, registry semantics, and metric envelopes.
- Stable public API mirrors and prelude ergonomics.
- FASTQ input assessment contracts and deterministic assessment helpers.

This crate does not own CLI parsing, workflow planning, runner execution,
product APIs, report rendering, environment provisioning, or stage-specific
business logic.

## Managed Operations

`docs/COMMANDS.md` is the SSOT for callable core operations, including:

- `canonicalize-json`
- `canonicalize-parameters-json`
- `canonicalize-truth-json`
- `canonical-json-bytes`
- `params-hash`
- `parameters-fingerprint`
- `input-fingerprint`
- `run-id-from-hashes`
- `parse-pipeline-id`
- `validate-pipeline-id`
- `parse-stage-id`
- `validate-stage-id`
- `parse-tool-id`
- `validate-tool-id`
- `validate-artifact-id`
- `validate-profile-id`
- `discover-fastq-files`
- `detect-fastq-path`
- `detect-gzip-path`
- `assess-input-dir`
- `write-input-assessment`
- `validate-execution-graph`
- `hash-execution-graph`
- `normalize-execution-graph`
- `topological-step-ids`
- `validate-execution-outputs`
- `query-run-index`
- `build-run-dir`
- `select-stage`
- `objective-spec`
- `parse-metric-id`
- `parse-derived-metric-id`
- `validate-metric-id`
- `validate-derived-metric-id`
- `metrics-schema-for-stage`

## Architecture

- `src/contract/` owns serialized contract families and validation rules.
- `src/foundation/` owns crate-private canonicalization, hashing, command specs,
  errors, invariants, measurement, and input assessment helpers that are exposed
  through stable public modules and prelude groups when downstream crates need
  them.
- `src/id_catalog/` owns canonical pipeline, stage, and tool constants.
- `src/ids/` owns typed identifiers, parsing, and domain models.
- `src/metrics/` owns metric ids, schemas, registry lookup, and metric payloads.
- `src/prelude/` owns grouped import ergonomics.
- `src/public_api/` owns curated stable surface mirrors.

## Allowed `pub` modules

- `contract`
- `id_catalog`
- `ids`
- `metrics`
- `prelude`
- `public_api`

## Documentation

The crate root intentionally has only this `README.md`. All other crate docs live
under `docs/`, with a 10-document allowance enforced by boundary tests:

- `docs/ARCHITECTURE.md`
- `docs/BOUNDARY.md`
- `docs/CHANGE_RULES.md`
- `docs/COMMANDS.md`
- `docs/CONTRACTS.md`
- `docs/CONTRACT_MAP.md`
- `docs/INVARIANTS.md`
- `docs/PUBLIC_API.md`
- `docs/SERIALIZATION.md`
- `docs/TESTS.md`

## Verification

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-core --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test semantics --no-default-features
```
