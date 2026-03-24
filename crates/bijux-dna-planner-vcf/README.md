# bijux-dna-planner-vcf

## What this crate does
Owns VCF plan graph assembly from domain stages and typed params.

## What it must not do (boundaries)
Must not execute tools, parse runtime metrics files, or own stage invocation internals.

## Effects & determinism guarantees
Plan output ordering is deterministic and idempotent for identical inputs.

## Public API / entrypoints
- `src/lib.rs`

## Key contracts it owns/consumes
Consumes domain-vcf stage/params contracts and emits planner-level contracts for API use.

## Artifacts / Contracts
- [crates/bijux-dna-planner-vcf/docs/INDEX.md](crates/bijux-dna-planner-vcf/docs/INDEX.md)
- [crates/bijux-dna-planner-vcf/docs/SCOPE.md](crates/bijux-dna-planner-vcf/docs/SCOPE.md)
- [crates/bijux-dna-planner-vcf/docs/ARCHITECTURE.md](crates/bijux-dna-planner-vcf/docs/ARCHITECTURE.md)
- [crates/bijux-dna-planner-vcf/docs/TESTS.md](crates/bijux-dna-planner-vcf/docs/TESTS.md)

## Failure modes
Missing required stages or unstable graph ordering breaks planner contracts.

## How to run its tests
- `cargo test -p bijux-dna-planner-vcf --test contracts`
- `cargo test -p bijux-dna-planner-vcf --test guardrails`
- `cargo test -p bijux-dna-planner-vcf`
- `tests/contracts.rs`
- `tests/guardrails.rs`
- `src/lib.rs`

## Where the docs live
All crate docs live under `crates/bijux-dna-planner-vcf/docs/`, indexed by [crates/bijux-dna-planner-vcf/docs/INDEX.md](crates/bijux-dna-planner-vcf/docs/INDEX.md).
