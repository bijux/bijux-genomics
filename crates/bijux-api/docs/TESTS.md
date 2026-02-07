# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- Each test file should be referenced here.

## Examples
- `tests/schema/api_stability.rs` → schema snapshot stability.

## Failure modes
- Missing test documentation causes drift and confusion.

## schema
- `tests/schema/api_stability.rs` — API response schemas are stable.
- `tests/schema/docs_schema_snapshots.rs` — `API.md` references schema names.

## surface
- `tests/surface/public_surface.rs` — public surface is curated.
- `tests/surface/public_policy.rs` — public policy contracts are enforced.
- `tests/surface/v1_guardrails.rs` — v1 module layout guardrails.

## roundtrip
- `tests/roundtrip/explain_roundtrip.rs` — explain output roundtrips.
- `tests/roundtrip/contract_spine.rs` — plan → manifest/report contract spine.
- `tests/roundtrip/contract_handshake.rs` — fixture handshake across plan/manifest/report.

## guardrails
- `tests/guardrails/policies.rs` — shared policy enforcement.
- `tests/guardrails/args_module.rs` — `request_args.rs` is the only args module.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
