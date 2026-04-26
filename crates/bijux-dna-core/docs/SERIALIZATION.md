# Serialization

## Canonical JSON Rules

- Object keys are sorted lexicographically at every JSON object level.
- JSON numbers are serialized through `serde_json` after canonical value
  normalization, so equivalent numeric JSON values produce deterministic bytes.
- Arrays preserve caller order because array order is contract data.
- Strings, including paths and URLs, are preserved as values. Callers must pass
  already-normalized path strings when path normalization is part of their
  contract.

## Hashing Inputs

- Contract version fields that are part of the serialized payload.
- Canonical JSON bytes produced by `contract::to_canonical_json_bytes`.
- Explicit fingerprint inputs passed to `foundation::hashing` helpers, such as
  normalized parameter JSON, declared input hashes, or file bytes.

## Enforcement

- `tests/contracts/surface/canonicalization.rs` verifies key ordering, numeric
  determinism, URL preservation, parent-segment preservation, and metrics schema
  lookup.
- `tests/contracts/execution/execution_plan_contract.rs` verifies execution
  plan canonical JSON round trips.
- `tests/contracts/identity/hashing_identity.rs` verifies parameter and input
  fingerprint stability.
