# bijux-dna-stages-vcf

`bijux-dna-stages-vcf` owns the VCF stage execution surface used by the
genomics workspace. It provides typed stage runners, VCF IO helpers, preflight
invariants, metrics parsers, wrapper checks, and the dispatch entrypoint for
VCF stage plans.

This crate is intentionally effectful. Unlike FASTQ and BAM stage-spec crates,
the current VCF crate owns product stage execution helpers and writes stage
artifacts. Command-line routing, API request handling, planner policy, runtime
scheduling, and environment provisioning still belong outside this crate.

## What this crate does

This crate owns the VCF stage execution surface, typed stage runners, VCF IO
helpers, preflight invariants, metrics parsers, wrapper checks, and dispatch for
VCF stage plans.

## Boundaries

This crate does not own CLI routing, API request handling, planner policy,
runtime scheduling, or environment provisioning.

## Public Surface

- `engine`: dispatch request/result types and `run_vcf_pipeline`.
- `pipeline`: typed VCF stage runner families.
- `invariants`: VCF preflight checks and normalized artifact generation.
- `metrics`: deterministic parser helpers for VCF metrics payloads.
- `vcf_io`: VCF validation, normalization, indexing, region, and overlap helpers.
- `stage_specs`: VCF stage registry metadata.

## Release Example

Run the release-surface example from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -q -p bijux-dna-stages-vcf --example vcf_release_surface
```

The example prints the governed VCF implemented-stage set and asserts presence
of filter, QC, and stats stages expected in release-facing workflows.

## Documentation

The crate keeps one root `README.md`. All other crate documentation lives under
`docs/` and is indexed from [docs/INDEX.md](docs/INDEX.md).

Key docs:

- [docs/COMMANDS.md](docs/COMMANDS.md): SSOT for operations managed by this crate.
- [docs/BOUNDARY.md](docs/BOUNDARY.md): ownership and forbidden surfaces.
- [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md): dependency graph rules.
- [docs/STAGE_CONTRACTS.md](docs/STAGE_CONTRACTS.md): VCF stage coverage.
- [docs/TESTS.md](docs/TESTS.md): local verification commands.

## Tests

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-vcf --no-default-features
```
