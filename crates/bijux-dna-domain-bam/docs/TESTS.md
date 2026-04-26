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

## Examples
- `tests/contracts/parsers/bam_parsers.rs` → fixture parsing assertions.
- `tests/contracts/stage_contract_snapshots.rs` → reviewed JSON contract snapshots.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
