# bijux-dna-db-ena

## What this crate does
Owns typed ENA query contracts, filereport decoding, and deterministic download planning and
execution for ENA-sourced corpus inputs.

## What it must not do (boundaries)
This crate does not plan pipelines, execute workflow stages, or own workspace path policy outside
caller-provided output roots.

## Effects & determinism guarantees
This crate deterministically resolves ENA metadata and download manifests from explicit query
inputs and declared output roots.

## Internal layout
- `src/public_api/`: curated stable surface
- `src/cli_entrypoint.rs`: binary launcher handoff
- `src/cli/`: binary-only argument parsing and command assembly
- `src/manifest_store.rs`: manifest persistence for the binary workflow
- `src/client/filereport/`: ENA filereport request and parsing contracts
- `src/download/`: download planning, runtime setup, task contracts, and file transfer logic
- `src/model/`: ENA query, manifest, record, and source-selection contracts

## Public API / entrypoints
Start with `PUBLIC_API.md` and `docs/ARCHITECTURE.md`. The library root keeps compatibility
exports while routing the stable surface through `src/public_api/mod.rs`.

## Key contracts it owns/consumes
- crate scope: `docs/SCOPE.md`
- architecture: `docs/ARCHITECTURE.md`
- test map: `docs/TESTS.md`
- doc index: `docs/INDEX.md`

## Artifacts / Contracts
Manifest schemas and download planning contracts are documented in `docs/ARCHITECTURE.md`,
`docs/SCOPE.md`, and `docs/INDEX.md`.

## Failure modes
- invalid ENA query descriptors or malformed filereport payloads
- manifest persistence failures at caller-provided output roots
- download transfer integrity and resume-policy violations

## How to run its tests
See `docs/TESTS.md` for the full map. The test tree is organized by enduring intent:
- `tests/boundaries.rs`: source-tree guardrails
- `tests/contracts/`: reserved for future ENA contract coverage beyond unit tests
- `tests/determinism/`: reserved for future reproducibility assertions
- `tests/schemas/`: reserved for future public-surface or persisted-schema checks
- `tests/guardrails.rs`: repository policy entrypoint

## Where the docs live
Primary docs live in `docs/INDEX.md`; test coverage maps live in `docs/TESTS.md`.
