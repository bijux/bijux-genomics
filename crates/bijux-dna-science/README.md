# bijux-dna-science

## What this crate does
`bijux-dna-science` is the science control plane for the genomics workspace. It
loads authored science specs, validates cross references, compiles deterministic
traceability rows, refreshes governed generated science outputs, and cuts immutable
science release bundles.

## What it must not do (boundaries)
It must not execute workflows, orchestrate stages, launch tools, invoke containers,
choose pipeline routes, or own runtime policy.

## Implemented science surface

- Authored science spec validation for sources, evidence, claims, assumptions,
  reasoning, decisions, bindings, and release manifests.
- FASTQ environment and container traceability rows.
- Closure, truth-delta, missing-prerequisite, download-backlog, and default-risk
  evidence rows for FASTQ tooling.
- Deterministic TSV/JSON rendering for committed generated outputs.
- Immutable release bundle writing under `artifacts/science-releases/`.

## Public entrypoints

- Library: `app::validate_workspace`, `app::build_workspace`, `app::trace_workspace`,
  `app::release_workspace`, `compile::compile_workspace`
- binary: `cargo run -p bijux-dna-science -- <command>`

## Key contracts it owns/consumes

Owns authored science spec validation, typed science identifiers, compiled evidence
row shape, governed generated science output parity, and science release bundle
contracts. Consumes repository science specs and governed upstream evidence tables.
It does not own runtime pipeline contracts.

## Governed Science Surfaces

- [science/specs/evidence/README.md](../../science/specs/evidence/README.md)
  is the authored evidence input surface
- [science/specs/releases/README.md](../../science/specs/releases/README.md)
  is the authored release-manifest input surface
- [science/generated/README.md](../../science/generated/README.md) is the
  committed generated-output boundary
- [science/generated/current/evidence/README.md](../../science/generated/current/evidence/README.md)
  inventories the row-level outputs refreshed by `build`
- [science/generated/indexes/README.md](../../science/generated/indexes/README.md)
  inventories the rolled-up index outputs refreshed by `build`

## Effects & determinism guarantees

`build` and `release` write deterministic outputs from authored specs and governed
upstream evidence. `validate`, `trace`, and `closure` report compiled state without
launching tools or mutating runtime data.

## How to run its tests

- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-science --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-science --test contracts --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-science --test fastq_environment_slice --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-science --test tsv_shape --no-default-features`

## Where the docs live

See [docs/INDEX.md](docs/INDEX.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md),
[docs/COMMANDS.md](docs/COMMANDS.md), [docs/SCHEMAS.md](docs/SCHEMAS.md),
[docs/BOUNDARY.md](docs/BOUNDARY.md), [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md),
[docs/PUBLIC_API.md](docs/PUBLIC_API.md),
[docs/TESTS.md](docs/TESTS.md), and [docs/VERSIONING.md](docs/VERSIONING.md).

## Local archive boundary

This crate may validate and report expected archive paths, but it must not treat local archive
payloads as handwritten SSOT. Review-owned truth stays under `science/specs/**`.
