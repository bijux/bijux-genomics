# bijux-dna-core

## What this crate does
Defines canonical IDs, schemas, and canonicalization rules for the entire workspace.

## Canonical entry
Start at `docs/INDEX.md`. The three most important docs are:
- `docs/CONTRACT_MAP.md`
- `docs/CONTRACT_VERSIONING.md`
- `docs/BOUNDARIES.md`

## Contract map (authoritative)
`docs/CONTRACT_MAP.md` is the single authoritative map of all core contracts and their locations.

## What is SSOT here
`bijux-dna-core` is the single source of truth for contract JSON shapes, canonical bytes, and their hashing inputs.
Core owns IDs + canonicalization + contract schema; nobody else defines IDs.

## Hashing & canonicalization guarantees
- Canonical JSON serialization is stable and deterministic.
- Hashes are defined over canonical bytes, not raw inputs.
- Ordering for canonicalization is explicit and enforced by tests.

## What it must not do (boundaries)
No planning, execution, or IO side effects beyond pure serialization helpers.

## What MUST NOT exist here (effects)
- No tool selection or command assembly.
- No filesystem effects beyond pure serialization helpers.
- No runtime execution, scheduling, or IO side effects.

## Start here in code
- `src/contract/execution/*` for execution graph and run contracts.
- `src/metrics/*` for metrics registry and schemas.

## Role in the stack
Upstream: none. Downstream: runtime, engine, planners, stages, analyze, benchmarks.

## Allowed `pub` modules
- `contract`
- `foundation`
- `id_catalog`
- `ids`
- `metrics`
- `prelude`

## Public API / entrypoints
See `docs/INDEX.md`, `docs/CONTRACTS.md`, `docs/PUBLIC_API.md`, `docs/INVARIANTS.md`, `docs/SERIALIZATION.md`, `docs/SSOT.md`, `docs/CONTRACT_VERSIONING.md`.

## Effects & determinism guarantees
Canonicalization and hashing are deterministic and enforced by snapshot tests.

## Key contracts it owns/consumes
Owns IDs, canonical JSON rules, and all core contract schemas.

## Artifacts / Contracts
See `docs/CONTRACT_MAP.md` and schema snapshots under `tests/snapshots/`.

## How to run its tests
See `docs/TESTS.md` for the canonical test map. Key tests:
- `tests/contracts/public_api.rs`
- `tests/contracts/docs_public_api.rs`
- `tests/contracts/contract_invariants.rs`

## Failure modes
Primary failures surface as contract snapshot mismatches or invariant violations.

## Where the docs live
Start at `docs/INDEX.md` and follow the core docs listed above.

## Stability
Contract and behavior changes follow `docs/CONTRACT_VERSIONING.md`.
