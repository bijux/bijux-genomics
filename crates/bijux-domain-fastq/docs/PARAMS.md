# PARAMS

Canonicalization normalizes:
- ordering of keys
- float formatting
- path normalization

This ensures stable hashing and comparisons.

## Checklist: add a new stage param
- Update param schema and canonicalization in `src/params/*`.
- Update invariant expectations in `src/invariants/*`.
- Update metric semantics in `src/metrics/*`.
- Refresh stage contract snapshots in `tests/contracts/stage_contract_snapshots.rs`.
