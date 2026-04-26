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
- `tests/boundaries/*` → architecture, purity, determinism, and policy guardrails.
- `tests/contracts/*` → stage contracts, manifest parity, SSOT ids, and public surface.
- `tests/determinism/*` → fixture stability and reproducibility checks.
- `tests/semantics/*` → retention semantics, params canonicalization, observability.
- `tests/semantics/invariants/*` → invariant specs and invariant enforcement.

## Examples
- `tests/contracts/stage_contract_snapshots.rs` → stage contract snapshots.
- `tests/contracts/domain_manifest_parity.rs` → source-manifest parity with crate catalogs.
- `tests/semantics/invariants/invariant_specs.rs` → invariant coverage and metric fixtures.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
