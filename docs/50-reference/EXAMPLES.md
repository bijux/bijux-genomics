# EXAMPLES

## What
Pointers to deterministic fixtures used as reference examples.

## Why
Examples make schema and contract expectations concrete.

## Non-goals
- Full tutorials.

## Contracts
- Example fixtures must be deterministic and versioned.

## Examples
Deterministic fixtures are stored in:
- `crates/bijux-dna-analyze/tests/fixtures`
- `crates/bijux-dna-pipelines/tests/snapshots`

These are used by tests to enforce stability.

## Failure modes
- Orphaned fixtures drift without review.
