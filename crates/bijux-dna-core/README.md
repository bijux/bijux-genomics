# bijux-dna-core

## What this crate does
Defines the canonical contracts, identifiers, metrics types, and deterministic foundation rules for the entire workspace.

## Canonical entry
Start at `crates/bijux-dna-core/docs/INDEX.md`. The three most important docs are:
- `crates/bijux-dna-core/docs/CONTRACT_MAP.md`
- `crates/bijux-dna-core/docs/CONTRACT_VERSIONING.md`
- `crates/bijux-dna-core/docs/BOUNDARIES.md`

## Contract map (authoritative)
`crates/bijux-dna-core/docs/CONTRACT_MAP.md` is the single authoritative map of all core contracts and their locations.

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
- `src/public_api/` for the curated stable surface.
- `src/contract/` for contract families and canonical serialization.
- `src/id_catalog/` for canonical identifier families partitioned into `pipeline/`, `stage/`, and `tool/`.
- `src/ids/` for typed identities and validators partitioned into `typed/` and `parsing/`.
- `src/prelude/` for stable import ergonomics grouped by contract, catalog, identity, foundation, and metrics source areas.

## Role in the stack
Upstream: none. Downstream: runtime, engine, planners, stages, analyze, benchmarks.

## Allowed `pub` modules
- `contract`
- `foundation`
- `id_catalog`
- `ids`
- `metrics`
- `public_api`
- `prelude`

## Public API / entrypoints
See `crates/bijux-dna-core/docs/INDEX.md`, `crates/bijux-dna-core/docs/CONTRACTS.md`, `crates/bijux-dna-core/docs/PUBLIC_API.md`, `crates/bijux-dna-core/docs/INVARIANTS.md`, `crates/bijux-dna-core/docs/SERIALIZATION.md`, `crates/bijux-dna-core/docs/SSOT.md`, `crates/bijux-dna-core/docs/CONTRACT_VERSIONING.md`.

## Effects & determinism guarantees
Canonicalization and hashing are deterministic and enforced by snapshot tests.

## Key contracts it owns/consumes
Owns IDs, canonical JSON rules, and all core contract schemas.

## Artifacts / Contracts
See `crates/bijux-dna-core/docs/CONTRACT_MAP.md` and schema snapshots under `tests/snapshots/`.

## How to run its tests
See `crates/bijux-dna-core/docs/TESTS.md` for the canonical test map. Key tests:
- `tests/boundaries.rs`
- `tests/contracts.rs`
- `tests/schemas.rs`
- `tests/semantics.rs`

## Failure modes
Primary failures surface as contract snapshot mismatches or invariant violations.

## Where the docs live
Start at `crates/bijux-dna-core/docs/INDEX.md` and follow the core docs listed above.

## Stability
Contract and behavior changes follow `crates/bijux-dna-core/docs/CONTRACT_VERSIONING.md`.
