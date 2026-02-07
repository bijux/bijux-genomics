# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Artifacts suite (`tests/artifacts/*`)
- `tests/artifacts/qa_artifact_contract.rs` → artifact contract checks.

## Support suite (`tests/support/*`)
- `tests/support/image_qa_support.rs` → support utilities and fixtures.

## Guardrails suite (`tests/guardrails/*`)
- `tests/guardrails/guardrails.rs` → dependency boundaries.
- `tests/guardrails/offline_policy.rs` → offline-by-default policy.

## Failure modes
- Missing test documentation causes drift and confusion.
