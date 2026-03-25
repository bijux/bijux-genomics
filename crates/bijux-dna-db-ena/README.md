# bijux-dna-db-ena

## What this crate does
`bijux-dna-db-ena` owns typed ENA query, parsing, and download primitives used to materialize corpus inputs.

## What it must not do (boundaries)
This crate must not perform pipeline planning, stage execution, or report rendering. It must not own host-path policy.

## Effects & determinism guarantees
Effects are limited to ENA network I/O and explicit filesystem writes under caller-provided output roots. URL normalization and download task planning are deterministic from input metadata.

## Public API / entrypoints
Core entrypoints are in `src/client.rs`, `src/model.rs`, and `src/download.rs`.

## Key contracts it owns/consumes
It owns ENA response parsing/normalization contracts and consumes workspace guardrails for boundary and docs compliance.

## Artifacts / Contracts
- [docs/INDEX.md](docs/INDEX.md)
- [docs/SCOPE.md](docs/SCOPE.md)
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/TESTS.md](docs/TESTS.md)
- [BOUNDARY.md](BOUNDARY.md)
- [PUBLIC_API.md](PUBLIC_API.md)

## Failure modes
Common failures: ENA endpoint/network errors, malformed filereport rows, and download integrity mismatches.

## How to run its tests
- `cargo test -p bijux-dna-db-ena`
- `cargo test -p bijux-dna-db-ena client::tests`
- `cargo test -p bijux-dna-db-ena download::tests`
- `cargo test -p bijux-dna-db-ena model::tests`
- `cargo test -p bijux-dna-db-ena --test guardrails`

## Where the docs live
Crate documentation lives in `docs/` and is indexed from [docs/INDEX.md](docs/INDEX.md).
