# SERIALIZATION

## Canonical JSON Rules
All contract JSON must be written via the canonical serializer in `contract/canonical`.

Rules:
- Keys are sorted lexicographically.
- Floats are normalized (no platform-dependent formatting).
- Paths are normalized to workspace-relative when applicable.
- Optional fields with default values are omitted unless explicitly required.

## Why
Canonicalization guarantees:
- Stable hashes for caching and reproducibility.
- Deterministic diffs across machines.
- Safe comparison in policy tests and snapshots.

## Forbidden
- Direct `serde_json::to_writer`/`to_string` for contract artifacts.
- Custom JSON writers outside the canonical module.

## Example
Use the canonicalizer helper provided by this crate (pseudo-code):

```
let bytes = bijux_core::contract::canonical::to_canonical_json_bytes(&value)?;
write_all(bytes);
```
