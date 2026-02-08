# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- Each test file should be referenced here.
- Fixtures under `tests/fixtures/*` back the contract snapshots and golden comparisons.

## Fixture → contract mapping
- `tests/fixtures/bench_artifacts/decision.json` → `docs/BENCH_FORMAT.md#decisionjson`
- `tests/fixtures/bench_artifacts/observations.jsonl` → `docs/BENCH_FORMAT.md#observationsjsonl`
- `tests/fixtures/bench_artifacts/summary.json` → `docs/BENCH_FORMAT.md#summaryjson`
- `tests/fixtures/bench_bundle/*` → `docs/BENCH_CONTRACT.md`
- `tests/fixtures/handshake/run_record.json` → `tests/contracts/contract_handshake.rs`

## Suite map
- `tests/contracts/*` → boundary, API surface, and schema contract checks.
- `tests/determinism/*` → deterministic ordering and snapshot stability.
- `tests/gate/*` → policy and gating invariants.

## Examples
- `tests/contracts/architecture.rs` → dependency boundary assertions.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
