# CONTRACTS

## What
This crate is the contract bible for the workspace. Contracts are the stable, serialized shapes used to plan, execute, and report runs. The canonical contract map is:

- `contract/execution/*` — `ExecutionGraph`, steps, edges, policy.
- `contract/run/*` — `RunManifest`, `RunRecord`, layout references.
- `contract/tooling/*` — tool identity, invocation, cache key.
- `contract/version.rs` — contract versioning rules.
- `metrics/*` — metrics envelope shapes and semantics.
- `ids.rs` — strongly-typed IDs (no raw `String` in public contracts).

## Why
Keeping the contracts here makes serialization stable and prevents drift across crates. Planners, runtime, engine, and analyze all depend on these shapes.

## Versioning
- Backward-compatible additions bump **minor**.
- Breaking field removals/renames bump **major**.
- All serialized artifacts embed `contract_version`.

## Canonicalization
All serialized contract JSON must pass through the canonical serializer in `contract/canonical`. Canonicalization guarantees:

- Stable key ordering.
- Normalized floats.
- Normalized, relative paths.
- Machine-independent hashing.

## Non-goals
- Tool execution, process spawning, or filesystem effects.
- Any domain-specific selection logic.

## See also
- `docs/SERIALIZATION.md`
- `docs/INVARIANTS.md`
- `docs/PUBLIC_API.md`
