# bijux-core

## Canonical entry
Start at `docs/INDEX.md`. The three most important docs are:
- `docs/CONTRACT_MAP.md`
- `docs/CONTRACT_VERSIONING.md`
- `docs/BOUNDARIES.md`

## What contracts exist
This crate defines the stable, serialized contracts for:
- Execution planning: `ExecutionGraph`, `ExecutionStep`, `ExecutionManifest`
- Run records and provenance: `RunRecordV1`, `RunMetadataV1`, `ScientificProvenanceV1`
- Tooling and registry types: `StageSpec`, `ToolManifest`, `ToolRegistry`
- Metrics registry and envelopes in `src/metrics/*`
- Strong IDs in `src/ids.rs`

## What is SSOT here
`bijux-core` is the single source of truth for contract JSON shapes, canonical bytes, and their hashing inputs. Other crates may build or consume these contracts, but they do not redefine them.

## Hashing & canonicalization guarantees
- Canonical JSON serialization is stable and deterministic.
- Hashes are defined over canonical bytes, not raw inputs.
- Ordering for canonicalization is explicit and enforced by tests.

## What MUST NOT exist here (effects)
- No tool selection or command assembly.
- No filesystem effects beyond pure serialization helpers.
- No runtime execution, scheduling, or IO side effects.

## Start here in code
- `src/contract/execution/*` for execution graph and run contracts.
- `src/metrics/*` for metrics registry and schemas.

## Role in the stack
Upstream: none. Downstream: runtime, engine, planners, stages, analyze, benchmarks.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/CONTRACTS.md`, `docs/PUBLIC_API.md`, `docs/INVARIANTS.md`, `docs/SERIALIZATION.md`, `docs/SSOT.md`, `docs/CONTRACT_VERSIONING.md`.

## How to run its tests
See `docs/TESTS.md` for the canonical test map.

## Stability
Contract and behavior changes follow `docs/CONTRACT_VERSIONING.md`.
