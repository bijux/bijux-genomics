# SERIALIZATION

## Canonical JSON Rules
- Keys sorted lexicographically.
- Floats normalized to a stable decimal representation.
- Paths normalized to workspace-relative when applicable.

## Hashing Inputs
- Contract version
- Canonical JSON bytes
- Normalized paths and floats

## Enforcement
- `tests/canonicalization.rs` verifies stable ordering and normalization.
- `tests/execution_plan_contract.rs` verifies canonical JSON usage.
