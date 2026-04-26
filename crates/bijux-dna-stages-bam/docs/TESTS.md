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
- `tests/boundaries/*` → layering, purity, and policy guardrails.
- `tests/contracts/*` → stage specs, registry, structure, and contract snapshots.
- `tests/contracts/observer/*` → observer parser fixtures under `tests/fixtures/observer/default/*`.
- `tests/determinism/*` → reproducibility and fixture stability checks.
- `tests/semantics/metrics/*` → metric completeness checks.

## Structure contract
The structure contract test enforces expected stage spec/observer layout to prevent
accidental drift that breaks contract snapshots.

## Examples
- `tests/contracts/observer/observer_parsers.rs` → observer fixture parsing.
- `tests/semantics/metrics/metrics_completeness.rs` → metric completeness coverage.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
