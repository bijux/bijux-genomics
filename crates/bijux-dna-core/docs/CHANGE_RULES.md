# Change Rules

These rules govern changes to `bijux-dna-core` contract, identifier, metric,
canonicalization, hashing, and public API surfaces.

## Breaking Changes

A change is breaking when existing downstream code, serialized data, canonical
bytes, hashes, or public imports can observe different behavior.

Breaking changes include:

- Renaming, removing, or changing the type or semantics of a public struct field.
- Renaming public modules or removing public re-exports.
- Changing identifier parsing rules in a way that rejects previously valid ids
  or accepts ambiguous ids.
- Changing canonical JSON key ordering, normalization, default handling, or hash
  inputs.
- Reordering arrays where order has contract meaning.
- Tightening validation to reject previously valid serialized contracts.
- Loosening validation in a way that allows ambiguous or unsafe states.
- Changing metric id syntax, schema ids, metric units, or derived metric meaning.
- Moving a callable operation to a different public path without preserving the
  old path or documenting a migration.
- Adding filesystem effects to an operation that was previously pure.

Breaking changes require an explicit contract review, affected docs, updated
tests, and snapshot updates in the same work item.

## Non-Breaking Changes

The following are normally non-breaking when existing behavior remains stable:

- Adding optional serialized fields with defaults and compatibility tests.
- Adding new public types under an existing public module.
- Adding new identifier constants without changing existing constants.
- Adding validation for inputs that were already invalid by documented contract.
- Clarifying docs without changing behavior.
- Adding a new managed operation when it is documented in `docs/COMMANDS.md`,
  tested through the closest suite, and exposed through an existing public
  module.

## Required Updates

Every behavior change must update the closest SSOT:

- Managed operations: `docs/COMMANDS.md`.
- Source/test layout: `docs/ARCHITECTURE.md` and boundary tests.
- Boundary or dependency rules: `docs/BOUNDARY.md` and boundary tests.
- Contract families and ownership: `docs/CONTRACTS.md` and `docs/CONTRACT_MAP.md`.
- Public modules and public surface: `docs/PUBLIC_API.md`, schema tests, and
  snapshots.
- Canonicalization/hashing rules: `docs/SERIALIZATION.md` and contract tests.
- Invariants: `docs/INVARIANTS.md` and the matching contract or semantic tests.
- Test ownership: `docs/TESTS.md`.

If a change touches more than one of these surfaces, update every affected SSOT
in the same commit or explain why the split is necessary in the handoff.

## Verification

Use the narrowest test suite for the changed surface. Public API or schema
changes require:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test schemas --no-default-features
```

Contract, hashing, or canonicalization changes require:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test contracts --no-default-features
```
