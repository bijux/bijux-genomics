# Change Rules

These rules describe how benchmark model contracts can change without silent
drift.

## Breaking Changes

A change is breaking when it changes a public record, public validation result,
schema id, deterministic ordering, gate decision field, comparison report field,
or statistical meaning that downstream crates can observe.

Examples:

- Renaming, removing, or changing the type of a public struct field.
- Changing `validate_suite`, `validate_observation`, `validate_summary`, or
  `validate_decision` from pass to fail, or fail to pass, for existing valid
  fixtures without an explicit contract reason.
- Reordering rationale traces, comparison diffs, graph nodes, or missing metric
  lists when the input is unchanged.
- Changing metric direction semantics, gate threshold behavior, or regression
  window interpretation.
- Changing bootstrap seeding or robust estimator formulas.

Breaking changes require review of affected docs, tests, snapshots, and callers
in the same work item.

## Non-Breaking Changes

The following are normally non-breaking when existing public behavior remains
stable:

- Adding validation tests for already-invalid inputs.
- Adding private helper modules under an existing boundary.
- Adding new public fields only when they have defaults and downstream
  serialization compatibility is proven.
- Clarifying docs without changing contracts.
- Adding a new managed model operation to `docs/COMMANDS.md` and the public API
  only when the operation is additive.

## Required Updates

Every behavior change must update the closest SSOT:

- Managed callable operations: `docs/COMMANDS.md`.
- Source and test layout: `docs/ARCHITECTURE.md` and boundary tests.
- Public exports: `docs/PUBLIC_API.md`, schema tests, and snapshots.
- Gate decisions: `docs/GATE_POLICY.md` and semantics tests.
- Determinism or randomness: `docs/DETERMINISM.md` and determinism tests.
- Statistical formulas: `docs/STATISTICS.md` and stats tests.
- Test ownership: `docs/TESTS.md`.

## Verification

Use the narrowest check for the changed surface before committing. For contract
or public API changes, run the affected integration test plus:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test schemas --no-default-features
```
