# Boundary

Owner: reference database resolution.

`bijux-dna-db-ref` is a deterministic, read-only reference metadata resolver. It
loads checked-in runtime and VCF catalog configuration, validates lock-backed
metadata, and projects reference, species, panel, and map contracts for VCF
planning.

## Owns

- Species alias, species authority, contig map, sex chromosome, and coverage
  profile resolution.
- Reference bank, reference bundle, provenance, organellar policy, and default
  reference-set resolution.
- Panel and genetic map catalog lookup, lock parsing, lock validation, and tool
  compatibility checks.
- Runtime provider traits and the default runtime-backed provider.

## Does Not Own

- Pipeline planning, stage execution, runner behavior, or environment setup.
- Network access, download execution, process spawning, or filesystem writes.
- CLI command routing or product API handlers.
- Reference data publication, mirror sync, or artifact generation.

## Allowed Effects

- Read-only TOML loading from repository-controlled `configs/runtime/*`,
  `configs/vcf/panels/*`, and `configs/vcf/maps/*`.
- Deterministic in-memory transformations into typed contracts.

## Forbidden Dependencies

This crate may depend on `bijux-dna-domain-vcf` for shared VCF species and
coverage contracts. It must not depend on CLI, API, planner, engine, runner,
runtime, environment, stage, benchmark, or data-download crates.

Dev-only guardrail dependencies are allowed for policy tests.

## Validation

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ref --test boundaries --no-default-features
```
