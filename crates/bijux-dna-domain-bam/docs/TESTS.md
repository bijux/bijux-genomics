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
- `tests/contracts/parsers/*` → parsing fixtures under `tests/fixtures/bam/default/*` and `tests/fixtures/tool_metrics/default/*`.
- `tests/contracts/*` → stage contracts, public surface, canonical serialization, and docs checks.
- `tests/semantics/invariants/*` → invariant specs and stage semantics.
- `tests/contracts/reference_suite/*` → reference suite coverage.
- `tests/determinism/*` → fixture and snapshot stability.
- `tests/boundaries/*` → purity and guardrail enforcement.
- `tests/support/mod.rs` → crate-local test helpers; shared helpers belong in `bijux-dna-testkit`.

## Examples
- `tests/contracts/parsers/bam_parsers.rs` → fixture parsing assertions.
- `tests/contracts/stage_contract_snapshots.rs` → reviewed JSON contract snapshots.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
Use `bijux-dna-testkit` for shared fixture and snapshot helpers. Keep crate-local support helpers small and focused.
