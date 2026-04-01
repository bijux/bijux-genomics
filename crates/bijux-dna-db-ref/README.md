# bijux-dna-db-ref

## What this crate does
Owns deterministic species/build reference resolution for VCF planning (`resolve_species_context`, `resolve_reference_bundle`) and runtime authority lookups (`species.toml`, `reference_bank.toml`).
Also owns genetic map bank lookups, sex/PAR policy, organellar policy, and default reference-set selection APIs.

## What it must not do (boundaries)
Must not execute tools, spawn processes, or perform runtime orchestration; it only resolves lock-backed metadata.

## Effects & determinism guarantees
Pure lookup behavior over checked-in config/lock material; no hidden side effects and deterministic outputs for identical inputs.

## Public API / entrypoints
Start with `PUBLIC_API.md`, `docs/ARCHITECTURE.md`, and `docs/TESTS.md`. The public surface is curated through `src/public_api/`, while runtime loading, models, providers, and lookup behavior live in dedicated namespaces under `src/`.

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
- Missing species/reference authority metadata.
- Contig/build mismatch at resolution time.

## How to run its tests
- `cargo test -p bijux-dna-db-ref`
- `cargo test -p bijux-dna-db-ref --test boundaries`
- `cargo test -p bijux-dna-db-ref --test contracts`
- `tests/guardrails.rs`
- `tests/boundaries.rs`
- `tests/contracts.rs`

## Where the docs live
All crate docs are in [`docs/`](docs/INDEX.md), with test guidance in [`docs/TESTS.md`](docs/TESTS.md).
