# Contract Versioning

## Purpose
Defines breaking vs non-breaking changes for bijux-core serialized contracts.

## Stability rules
Breaking changes require explicit approval, version bump, and snapshot updates.

### JSON shape
Breaking changes include:
- Renaming fields.
- Removing required fields.
- Changing field types or semantics.
- Changing required/default behavior that alters serialized output.

Non-breaking changes include:
- Adding optional fields with defaults.
- Adding new enum variants gated by version checks.

### Ordering
Canonical ordering is part of the contract:
- Key ordering for canonical JSON is fixed and must not change.
- Array ordering is semantic; reordering breaks hashes and is breaking.

### Hashing & canonicalization
Breaking changes include:
- Changing canonicalization rules.
- Changing hashing inputs or algorithms.
- Changing how defaults are normalized before hashing.

### Validation rules
Breaking changes include:
- Tightening validation to reject previously valid serialized inputs.
- Loosening validation in ways that allow ambiguous or unsafe states.

## Enforcement
- Snapshot tests and schema fixtures must be updated together with version bumps.
- Tests in `docs/TESTS.md` map to these invariants.
