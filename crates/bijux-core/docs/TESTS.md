# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- Each test file should be referenced here.

## Contract invariants
- `tests/contract/canonicalization.rs` → canonical JSON ordering and normalization.
- `tests/contract/execution_graph_validate.rs` → graph validation and acyclic guarantees.
- `tests/contract/execution_graph_purity.rs` → execution graph purity invariants.
- `tests/contract/execution_plan_contract.rs` → execution plan contract behavior.
- `tests/contract/run_index.rs` → run index schema parsing and filtering.
- `tests/contract/run_metadata.rs` → run metadata and provenance schema parsing.
- `tests/contract/sanity.rs` → cross-crate fixture parsing for core contract schemas.
- `tests/contract.rs` → suite entrypoint for contract tests.

## Public surface & boundaries
- `tests/public_api_lock.rs` → public module surface matches `docs/PUBLIC_API.md`.
- `tests/public_module_tree.rs` → lib.rs public module snapshot.
- `tests/public_surface.rs` → public surface scope checks.
- `tests/public_surface_lock.rs` → public surface snapshot lock.
- `tests/guardrails.rs` → boundary checks and crate layering.
- `tests/core_scope_guardrail.rs` → scope guardrails.
- `tests/layering.rs` → core layering and dependency boundaries.
- `tests/prelude_snapshot.rs` → prelude export surface snapshot.

## Metrics
- `tests/metrics/registry.rs` → metrics registry completeness.
- `tests/metrics.rs` → suite entrypoint for metrics tests.

## IDs
- `tests/ids/smoke.rs` → ID type smoke tests.
- `tests/ids.rs` → suite entrypoint for ID tests.

## Input assessment
- `tests/input_assessment.rs` → fastq input discovery and layout assessment behavior.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
