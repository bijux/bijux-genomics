# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- `tests/docs_lightweight.rs` → docs anchors and minimal usage examples.
- `tests/public_api_surface.rs` → public surface lock.
- `tests/dev_dep_boundary.rs` → testkit stays a dev-only dependency.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
