# bijux-dna-stages-vcf

## What this crate does
Owns VCF stage wiring, metrics schema typing, and parser/smoke support metadata.

## What it must not do (boundaries)
Must not own pipeline profile selection, domain ID authority, or execution runtime policy.

## Effects & determinism guarantees
Stage metadata and parser outputs are deterministic for fixed input fixtures.

## Public API / entrypoints
- `src/lib.rs`
- `src/metrics.rs`
- `src/stage_specs.rs`

## Key contracts it owns/consumes
Consumes domain-vcf stages and owns stage-level metrics schema compatibility checks.

## Artifacts / Contracts
- [crates/bijux-dna-stages-vcf/docs/INDEX.md](crates/bijux-dna-stages-vcf/docs/INDEX.md)
- [crates/bijux-dna-stages-vcf/docs/SCOPE.md](crates/bijux-dna-stages-vcf/docs/SCOPE.md)
- [crates/bijux-dna-stages-vcf/docs/ARCHITECTURE.md](crates/bijux-dna-stages-vcf/docs/ARCHITECTURE.md)
- [crates/bijux-dna-stages-vcf/docs/TESTS.md](crates/bijux-dna-stages-vcf/docs/TESTS.md)

## Failure modes
Unsupported parser/schema pairings or missing fixtures break stage contracts.

## How to run its tests
- `cargo test -p bijux-dna-stages-vcf --test contracts`
- `cargo test -p bijux-dna-stages-vcf --test guardrails`
- `cargo test -p bijux-dna-stages-vcf`
- `tests/contracts.rs`
- `tests/guardrails.rs`
- `src/stage_specs.rs`

## Where the docs live
All crate docs live under `crates/bijux-dna-stages-vcf/docs/`, indexed by [crates/bijux-dna-stages-vcf/docs/INDEX.md](crates/bijux-dna-stages-vcf/docs/INDEX.md).
