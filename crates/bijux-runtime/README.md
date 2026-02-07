# bijux-runtime

## What this crate does
Defines runtime contracts and recording: `RunLayout`, `RunManifest`, `RunRecord`, and `Provenance` plus recorder interfaces.

## What it must not do (boundaries)
Must not execute tools or perform heavy effects. It only writes files under the run layout.

## Public API / entrypoints
Runtime contracts and recorder interfaces documented in `docs/RUNTIME_CONTRACT.md` and `docs/EVENTS.md`.

## Key contracts it owns/consumes
Owns run layout/record/provenance schemas; consumes core contract IDs. See `docs/INDEX.md`.

## Effects & determinism guarantees
Filesystem writes only under layout; path derivation is deterministic. See `docs/BOUNDARY.md` and `tests/run_layout_contract.rs`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/reference_example.rs`, `tests/runtime_schema_snapshots.rs`, `tests/manifest_integrity.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then `docs/RUNTIME_CONTRACT.md` and `docs/GLOSSARY.md`.

## Artifacts / Contracts
Produces run layout trees and manifests/records for consumption by analyze/bench.

## Failure modes
Schema mismatches and missing fields are caught by snapshot and integrity tests.

## Stability
Schema changes are versioned and snapshot-tested per `docs/CHANGE_RULES.md`.
