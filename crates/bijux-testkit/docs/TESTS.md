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
- `tests/guardrails.rs` → boundary checks and dependency rules.
- `tests/public_api_surface.rs` → public API stays tiny.
- `tests/public_api_snapshot.rs` → public API snapshot lock.
- `tests/dev_dep_boundary.rs` → testkit is dev-only and isolated.

## Fixture guidance
See `docs/ADD_FIXTURE.md` and `docs/FIXTURE_STANDARDS.md`.

## Failure modes
- Missing test documentation causes drift and confusion.
