# bijux-dna-db-ref

## What this crate does
Owns deterministic species/build reference resolution for VCF planning (`resolve_species_context`, `resolve_reference_bundle`).

## What it must not do (boundaries)
Must not execute tools, spawn processes, or perform runtime orchestration; it only resolves lock-backed metadata.

## Effects & determinism guarantees
Pure lookup behavior over checked-in config/lock material; no hidden side effects and deterministic outputs for identical inputs.

## Public API / entrypoints
- `src/lib.rs`
- `PUBLIC_API.md`

## Key contracts it owns/consumes
Owns reference governance contracts and consumes `bijux-dna-domain-vcf` species/reference types.

## Artifacts / Contracts
- [docs/INDEX.md](docs/INDEX.md)
- [docs/SCOPE.md](docs/SCOPE.md)
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/TESTS.md](docs/TESTS.md)

## Failure modes
- Unknown species/build alias.
- Missing/invalid reference bundle lock metadata.
- Contig/build mismatch at resolution time.

## How to run its tests
- `cargo test -p bijux-dna-db-ref`
- `tests/guardrails.rs`
- `tests/boundaries/README.md`
- `tests/contracts/README.md`
- `tests/determinism/README.md`
- `tests/schemas/README.md`

## Where the docs live
All crate docs are in [`docs/`](docs/INDEX.md), with test guidance in [`docs/TESTS.md`](docs/TESTS.md).
