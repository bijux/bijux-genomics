# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Contracts
- Each test file should be referenced here.
- v1 endpoint tests are named `v1_fastq_*`, `v1_bam_*`, or `v1_cross_*`.

## v1_cross
- `tests/v1_cross_api_stability.rs` — API response schemas are stable.
- `tests/v1_cross_docs_schema_snapshots.rs` — `API.md` references schema names.
- `tests/v1_cross_public_surface.rs` — public surface is curated.
- `tests/v1_cross_public_policy.rs` — public policy contracts are enforced.
- `tests/v1_cross_guardrails.rs` — v1 module layout guardrails.
- `tests/v1_cross_explain_roundtrip.rs` — explain output roundtrips.
- `tests/v1_cross_contract_spine.rs` — plan → manifest/report contract spine.
- `tests/v1_cross_contract_handshake.rs` — fixture handshake across plan/manifest/report.

## guardrails
- `tests/guardrails/policies.rs` — shared policy enforcement.
- `tests/guardrails/args_module.rs` — `request_args.rs` is the only args module.
- `tests/guardrails.rs` — guardrail harness for bijux-dna-api.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
