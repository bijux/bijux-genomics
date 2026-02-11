# Manifest Schema Migration Policy

## Scope
- `bijux.profile_manifest.v1`
- run manifest/lock artifacts written by runner/runtime/report layers

## Version pinning
- Every manifest payload must include an explicit `schema_version`.
- Schema version changes are intentional and reviewed.

## Breaking vs non-breaking
- Breaking:
  - remove required fields
  - change field semantics
  - change hash inputs
- Non-breaking:
  - add optional fields
  - add new metadata sections that do not affect semantic meaning

## Snapshot impact
- If schema version changes, snapshot updates are expected.
- PR must include a migration note describing:
  - old version
  - new version
  - compatibility behavior

## Hash contract
- Profile hash is derived from canonicalized profile manifest only.
- Non-semantic ordering changes must not alter hash.
