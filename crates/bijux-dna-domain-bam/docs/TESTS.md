# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- Each test file should be referenced here.

## Suite map
- `tests/parsers/*` → parsing fixtures under `tests/fixtures/bam/*`.
- `tests/contracts/*` → stage contracts, public surface, and docs checks.
- `tests/invariants/*` → invariant specs and phase semantics.
- `tests/reference_suite/*` → reference suite coverage.

## Examples
- `tests/parsers/bam_parsers.rs` → fixture parsing assertions.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
