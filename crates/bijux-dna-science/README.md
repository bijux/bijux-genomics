# bijux-dna-science

## What this crate does
Compiles authored science specs into deterministic traceability outputs and science release bundles.

## What it must not do (boundaries)
No workflow execution, no stage orchestration, and no direct tool launching.

## First implemented slice
FASTQ environment and container support:

- admitted stage-tool surface
- governed defaults
- planned tools kept outside the closed runtime surface
- environment and container references backing those decisions
- local evidence archive planning for non-shareable papers and upstream source
  material under governed science archive paths

## Public entrypoints
- library: `app::validate_workspace`, `app::build_workspace`, `app::trace_workspace`,
  `app::release_workspace`
- binary: `cargo run -p bijux-dna-science -- <command>`

## Key contracts it owns/consumes
Owns the authored science spec validation, traceability output shape, and release bundle contract.
Consumes repository science specs and governed archive manifests; it does not own runtime pipeline
contracts.

## Effects & determinism guarantees
Build and release commands write deterministic generated science outputs from authored specs.
Validation and trace commands report contract state without launching tools or mutating runtime data.

## How to run its tests
- `cargo test -p bijux-dna-science`
- `cargo test -p bijux-dna-science --test guardrails`
- `cargo test -p bijux-dna-science --test contracts`

Primary test files:
- `tests/guardrails.rs`
- `tests/contracts.rs`
- `tests/fastq_environment_slice.rs`
- `tests/tsv_shape.rs`

## Where the docs live
See [docs/INDEX.md](docs/INDEX.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md),
[docs/COMMANDS.md](docs/COMMANDS.md), [docs/SCHEMAS.md](docs/SCHEMAS.md),
[docs/BOUNDARY.md](docs/BOUNDARY.md), [docs/PUBLIC_API.md](docs/PUBLIC_API.md),
[docs/TESTS.md](docs/TESTS.md), and [docs/VERSIONING.md](docs/VERSIONING.md).

## Local Archive Boundary

This crate may validate and report expected archive paths, but it must not treat local archive
payloads as handwritten SSOT. Review-owned truth stays under `science/specs/**`.
