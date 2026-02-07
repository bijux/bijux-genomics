# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- `tests/registry/*` → registry ordering and snapshot coverage.
- `tests/defaults/*` → defaults ledger ordering and override precedence.
- `tests/profiles/*` → profile completeness and contract checks.
- `tests/guardrails/*` → policy guardrails (including declarative pipelines).

## Mapping
- `tests/registry/pipeline_registry_snapshot.rs` → registry snapshot stability.
- `tests/registry/docs_registry_order.rs` → docs ordering matches registry.
- `tests/defaults/defaults_ledger.rs` → defaults ledger formatting + canonical JSON.
- `tests/defaults/override_precedence.rs` → override precedence contract.
- `tests/profiles/pipeline_completeness.rs` → profile completeness invariants.
- `tests/profiles/pipeline_contract.rs` → plan contract handshake fixtures.
- `tests/profiles/pipeline_ids_unique.rs` → unique profile ids.
- `tests/profiles/pipeline_id.rs` → pipeline id validation.
- `tests/profiles/profiles.rs` → profile coverage.
- `tests/guardrails/guardrails.rs` → policy guardrails.
- `tests/guardrails/no_stage_contracts.rs` → declarative pipeline enforcement.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
