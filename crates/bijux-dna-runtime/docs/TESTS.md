# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts suite (`tests/contracts/*`)
- `tests/boundaries/guardrails.rs` → privacy boundaries and crate guardrails.
- `tests/contracts/manifest_integrity.rs` → manifest integrity and required fields.
- `tests/contracts/run_layout_contract.rs` → run layout invariants.
- `tests/contracts/canonical_writer.rs` → canonical JSON writer enforced for runtime emits.
- `tests/contracts/docs_layout.rs` → documentation and API map stay aligned with the crate tree.

## Schema suite (`tests/schemas/*`)
- `tests/schemas/schema/runtime_schema_snapshots.rs` → schema stability snapshots.

## Reference suite (`tests/contracts/reference/*`)
- `tests/contracts/reference/reference_example.rs` → end-to-end reference story.
- `tests/contracts/reference/docs_reference_example.rs` → documentation coverage for the reference story.

## Stability, integrity, privacy coverage
- Stability: `runtime_schema_snapshots.rs`, `canonical_writer.rs`.
- Integrity: `manifest_integrity.rs`, `run_layout_contract.rs`.
- Privacy boundaries: `guardrails.rs`.

## Failure modes
- Missing test documentation causes drift and confusion.
