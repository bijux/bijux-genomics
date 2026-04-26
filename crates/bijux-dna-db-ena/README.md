# bijux-dna-db-ena

This crate follows repository governance documentation. `README.md` and
`README.md`; re-read those files before editing this child
repository and before committing.

## Scope

`bijux-dna-db-ena` owns typed ENA query contracts, ENA filereport decoding,
deterministic download planning, ENA file transfer helpers, and a small helper
binary for direct query/download workflows.

This crate does not plan pipelines, execute workflow stages, own corpus command
UX in `bijux-dna`, or manage workspace path policy outside caller-provided
output and manifest paths.

## Managed Commands

`docs/COMMANDS.md` is the SSOT for callable operations. The helper binary owns:

- `query`
- `download`

The library surface also owns ENA filereport parsing, query validation, record
URL selection, download task planning, and task execution helpers documented in
`docs/COMMANDS.md`.

## Internal layout

- `src/public_api/`: curated stable surface
- `src/cli_entrypoint.rs`: binary launcher handoff
- `src/cli/`: binary-only argument parsing and command assembly
- `src/manifest_store.rs`: manifest persistence for the binary workflow
- `src/client/filereport/`: ENA filereport request and parsing contracts
- `src/download/`: download planning, runtime setup, task contracts, and file transfer logic
- `src/model/`: ENA query, manifest, record, and source-selection contracts

## Documentation

The crate root intentionally has only this `README.md`. All other crate docs
live under `docs/`, with a 10-document allowance enforced by boundary tests:

- `docs/ARCHITECTURE.md`
- `docs/BOUNDARY.md`
- `docs/CHANGE_RULES.md`
- `docs/COMMANDS.md`
- `docs/CONTRACTS.md`
- `docs/DEPENDENCIES.md`
- `docs/INDEX.md`
- `docs/PUBLIC_API.md`
- `docs/SCOPE.md`
- `docs/TESTS.md`

## Verification

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-db-ena --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ena --no-default-features
```
