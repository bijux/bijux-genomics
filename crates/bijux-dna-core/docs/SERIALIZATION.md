# Serialization

## Canonical JSON Operations

The public helpers have distinct contracts:

- `contract::canonical::canonicalize_json_value` sorts object keys recursively.
  It preserves array order and scalar values, including string and numeric
  representations. It does not normalize paths or read environment variables.
- `contract::canonical::parameters_json_canonicalization` and
  `contract::canonical::canonicalize_truth_json` sort keys and apply the shared
  numeric, path and privacy normalization described below.
- `contract::canonical::to_canonical_json_bytes` serializes through
  `canonicalize_truth_json` and then `serde_json`. It does not serialize raw
  strings without transformation.

Every operation preserves array order. Object key ordering does not make an
array a set or make permutations equivalent. Numeric normalization retains
integer values and uses `serde_json`'s floating-point number representation;
it does not promise that integer and floating-point representations such as
`1` and `1.0` have identical bytes.

### Path and privacy normalization

Truth and parameter normalization recurse through JSON values:

- String values containing `/` or `\` and not matching the serializer's
  `scheme://` guard use lexical `std::path` component normalization. Current
  host path rules remove `.` and resolve removable `..` components, preserving
  leading parent components in relative paths.
- Absolute paths retain a suffix beginning with the first `bench`,
  `run_artifacts`, `artifacts` or `runs` component when one exists. Otherwise
  they retain the final component. This transformation is lexical: it does
  not resolve symlinks or inspect filesystem contents.
- Other string values use privacy substitutions. Occurrences of the value of
  `USER` (or `USERNAME` when `USER` cannot be read) become `<user>`; occurrences
  of `HOSTNAME` become `<host>`. Non-space dotted strings whose path extension
  is `local` become `<host>`.
- The URI guard skips path normalization, but does not bypass privacy
  substitutions. URI and scientific-label strings are therefore not promised
  to remain byte-identical when they match privacy inputs.

These are the existing privacy and portability rules for truth bytes and
parameter identity. Determinism requires the same input and normalization
context, including the relevant environment values and host path rules.
Callers that need raw value preservation must distinguish that requirement
from truth serialization. Changing these transformations or replacing the
truth serializer with the key-only helper changes observable bytes and hashes
and requires the compatibility review in `CHANGE_RULES.md`.

### Execution graph array order

`ExecutionGraph::new` and `normalize` sort steps by step ID and stably sort edges
by source and target step IDs. Distinct artifact bindings with equal source
and target IDs retain their input order. Graph validation distinguishes the
full source, target, output-artifact and input-artifact tuple; it does not merge
those bindings.

`ExecutionGraph::hash` serializes the stored graph arrays through truth
canonicalization. Deserialization does not implicitly reorder those arrays.
The plan manifest's graph identity also includes the stored edge array. The
existing contract does not promise equal hashes for permutations of distinct
bindings with equal endpoints. Reordering them must not be treated as a
byte-preserving change merely because topology is unchanged.

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

- `tests/contracts/surface/canonicalization.rs` checks key ordering, numeric
  representation, one URL vector under the running environment, leading parent
  segments, and metrics schema lookup. It does not establish independence from
  privacy environment values.
- `tests/contracts/execution/execution_plan_contract.rs` verifies execution
  plan canonical JSON round trips.
- `tests/contracts/identity/hashing_identity.rs` verifies parameter and input
  fingerprint stability.
- `tests/semantics/input_assessment.rs` verifies input assessment persistence
  and path discovery behavior.
