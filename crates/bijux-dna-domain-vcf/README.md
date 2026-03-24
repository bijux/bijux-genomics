# bijux-dna-domain-vcf

## What this crate does
Owns VCF domain stage IDs, typed params, and schema-versioned serialization contracts.

## What it must not do (boundaries)
Must not execute tools, perform runtime orchestration, or embed runner behavior.

## Effects & determinism guarantees
Effects are pure data transforms with deterministic serde payloads and stable schema versions.

## Public API / entrypoints
- `src/lib.rs`
- `src/params/mod.rs`

## Key contracts it owns/consumes
Owns VCF param typing contracts and consumes workspace ID/catalog contracts.

## Artifacts / Contracts
- [crates/bijux-dna-domain-vcf/docs/INDEX.md](crates/bijux-dna-domain-vcf/docs/INDEX.md)
- [crates/bijux-dna-domain-vcf/docs/SCOPE.md](crates/bijux-dna-domain-vcf/docs/SCOPE.md)
- [crates/bijux-dna-domain-vcf/docs/ARCHITECTURE.md](crates/bijux-dna-domain-vcf/docs/ARCHITECTURE.md)
- [crates/bijux-dna-domain-vcf/docs/TESTS.md](crates/bijux-dna-domain-vcf/docs/TESTS.md)

## Failure modes
Schema drift, missing version fields, or invalid param roundtrips break contracts.

## How to run its tests
- `cargo test -p bijux-dna-domain-vcf --test contracts`
- `cargo test -p bijux-dna-domain-vcf --test guardrails`
- `cargo test -p bijux-dna-domain-vcf`
- `tests/contracts.rs`
- `tests/guardrails.rs`
- `src/params/mod.rs`

## Where the docs live
All crate docs live under `crates/bijux-dna-domain-vcf/docs/`, indexed by [crates/bijux-dna-domain-vcf/docs/INDEX.md](crates/bijux-dna-domain-vcf/docs/INDEX.md).
