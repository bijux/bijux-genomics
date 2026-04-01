# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts suite (`tests/contracts/*`)
- `tests/contracts/artifacts/qa_artifact_contract.rs` → artifact contract checks.
- `tests/contracts/qa_contracts.rs` → shared QA contract coverage, including support fixtures.

## Boundaries suite (`tests/boundaries/*`)
- `tests/boundaries/architecture.rs` → source tree contract.
- `tests/boundaries/guardrails/guardrails.rs` → dependency boundaries.
- `tests/boundaries/guardrails/offline_guardrails.rs` → offline-by-default policy.

## Determinism suite (`tests/determinism/*`)
- `tests/determinism/fixture_stability.rs` → stable fixture output checks.

## Support fixtures
- `tests/support/image_qa_support.rs` → support fixtures reused by contract tests.

## Failure modes
- Missing test documentation causes drift and confusion.
