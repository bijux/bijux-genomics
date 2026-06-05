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
- `tests/boundaries.rs` → boundary suite aggregator for ownership guardrails and purity.
- `tests/boundaries/architecture.rs` → root layout, docs allowance, source tree, and Markdown placement.
- `tests/boundaries/commands.rs` → command-free library boundary and `docs/COMMANDS.md`.
- `tests/boundaries/dependencies.rs` → direct dependency graph and forbidden execution-layer edges.
- `tests/contracts.rs` → contract suite aggregator for serialized API, stage contracts, parsers, docs, snapshots, and public surface.
- `tests/contracts/public_surface.rs` → exported module and `docs/PUBLIC_API.md` alignment.
- `tests/determinism.rs` → determinism suite aggregator for fixture and snapshot stability.
- `tests/semantics.rs` → semantic suite aggregator for invariant behavior.
- `tests/contracts/parsers/bam_tool_parsers.rs` and `tests/contracts/parsers/new_tool_metrics.rs` → parsing fixtures under `tests/fixtures/bam/default/*` and `tests/fixtures/tool_metrics/default/*`.
- `tests/contracts/parsers/raw_fixture_bank.rs` → governed raw BAM parser fixtures under `tests/fixtures/bench/parsers/bam/<stage>/<tool>/`.
- `tests/contracts/*` → stage contracts, public surface, canonical serialization, and docs checks.
- `tests/semantics/invariants/*` → invariant specs and stage semantics.
- `tests/contracts/reference_suite/*` → reference suite coverage.
- `tests/determinism/*` → fixture and snapshot stability.
- `tests/boundaries/*` → purity and guardrail enforcement.
- `tests/support/mod.rs` → crate-local test helpers; shared helpers belong in `bijux-dna-testkit`.
- `tests/snapshots/*` → reviewed stage contract snapshots.
- `tests/fixtures/*` → small deterministic BAM/reference/tool-metric fixtures.

The raw BAM parser bank for benchmark-facing observer fixtures lives at
`tests/fixtures/bench/parsers/bam/<stage>/<tool>/` in the repository root.
Domain BAM parser contracts consume that bank directly so raw tool outputs and expected normalized
JSON stay governed in one location.

## Examples
- `tests/contracts/parsers/bam_parsers.rs` → fixture parsing assertions.
- `tests/contracts/stage_contract_snapshots.rs` → reviewed JSON contract snapshots.

## Failure modes
- Missing test documentation causes drift and confusion.
- Markdown documentation under `tests/` is forbidden; test taxonomy belongs here.

## Testkit patterns
Use `bijux-dna-testkit` for shared fixture and snapshot helpers. Keep crate-local support helpers small and focused.
