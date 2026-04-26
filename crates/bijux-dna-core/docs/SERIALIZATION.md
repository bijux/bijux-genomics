# Serialization

## Canonical JSON Rules

- Object keys are sorted lexicographically at every JSON object level.
- JSON numbers are serialized through `serde_json` after canonical value
  normalization, so equivalent numeric JSON values produce deterministic bytes.
- Arrays preserve caller order because array order is contract data.
- Strings, including paths and URLs, are preserved as values. Callers must pass
  already-normalized path strings when path normalization is part of their
  contract.
- `contract::canonical::canonicalize_json_value` is the public value
  canonicalizer.
- `contract::canonical::to_canonical_json_bytes` is the public byte serializer
  for contract artifacts.
- `contract::canonical::parameters_json_canonicalization` is the parameter
  normalization path used before parameter hashing.

## Hashing Inputs

- Contract version fields that are part of the serialized payload.
- Canonical JSON bytes produced by
  `contract::canonical::to_canonical_json_bytes`.
- Explicit fingerprint inputs passed to `foundation::hashing` helpers, such as
  normalized parameter JSON, declared input hashes, or file bytes.
- `input_fingerprint` and `run_id_from_hashes` sort and deduplicate input
  hashes before hashing, so caller order does not affect identity.

## Filesystem Payloads

- `write_input_assessment` writes pretty JSON for human review, but the payload
  shape remains governed by `InputAssessmentV1`.
- Input assessment file paths are recorded as supplied by discovery. Core does
  not rewrite absolute/relative path semantics or resolve symlinks.
- Run-index helpers read newline-delimited `RunIndexLine` JSON. They query typed
  records but do not publish, compact, or rewrite index files.

## Enforcement

- `tests/contracts/surface/canonicalization.rs` verifies key ordering, numeric
  determinism, URL preservation, parent-segment preservation, and metrics schema
  lookup.
- `tests/contracts/execution/execution_plan_contract.rs` verifies execution
  plan canonical JSON round trips.
- `tests/contracts/identity/hashing_identity.rs` verifies parameter and input
  fingerprint stability.
- `tests/semantics/input_assessment.rs` verifies input assessment persistence
  and path discovery behavior.
