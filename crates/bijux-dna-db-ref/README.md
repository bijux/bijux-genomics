# bijux-dna-db-ref

This crate follows repository governance documentation. `README.md` and
`README.md`; re-read those files before editing this child
repository and before committing.

## Scope

`bijux-dna-db-ref` owns deterministic species/build reference resolution for
VCF planning and runtime provenance. It resolves species contexts, reference
bundles, reference banks, genetic maps, panels, sex/PAR policy, organellar
policy, and default reference sets from checked-in config and lock material.

This crate does not execute tools, spawn processes, perform network access, or
own runtime orchestration.

## Managed Operations

`docs/COMMANDS.md` is the SSOT for callable resolver operations, including:

- `resolve-species-alias`
- `resolve-species-context`
- `resolve-reference-bundle`
- `resolve-panel`
- `resolve-map`
- `validate-imputation-tool-compatibility`
- `ref-service`

## Architecture

- `src/catalog/` owns panel/map catalog records, lock records, and compatibility
  policy.
- `src/model/` owns species authority and reference asset contracts.
- `src/providers/` owns runtime-facing provider traits and the default runtime
  implementation.
- `src/resolution/` owns pure lookup behavior, lock validation, contig
  normalization, and compatibility checks.
- `src/runtime_config/` owns repository path discovery, TOML loading, and config
  DTOs.
- `src/public_api/` owns stable re-exports.

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
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-db-ref --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ref --no-default-features
```
