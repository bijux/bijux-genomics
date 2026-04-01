# Tests

## Intent
The test tree is organized by what each suite protects, not by implementation history.

## Suite map
- `tests/boundaries.rs`: architecture tree checks, public-surface limits, ownership guardrails,
  and layering rules
- `tests/contracts.rs`: report artifacts, facts, dashboard exports, loader behavior, public API
  compatibility, pipeline handoff rules, and performance/privacy contracts
- `tests/determinism.rs`: canonical fixture ordering and stable serialization guarantees
- `tests/schemas.rs`: SQLite migrations, latest-schema reads, deterministic query behavior, and
  SQL query linting
- `tests/semantics.rs`: compare, ranking, and decision-trace behavior

## Important directories
- `tests/fixtures/`: durable fixture inputs for contract and determinism coverage
- `tests/snapshots/`: blessed text, HTML, JSON, and schema snapshots
- `tests/contracts/`: split into `core/`, `dashboard/`, `facts/`, `fastq/`, `metrics/`,
  `pipeline/`, and `report/`

## Regenerating affected coverage
- report and pipeline contracts: `cargo test -p bijux-dna-analyze --test contracts -j 1`
- boundaries and tree guardrails: `cargo test -p bijux-dna-analyze --test boundaries -j 1`
- SQLite schema coverage: `cargo test -p bijux-dna-analyze --test schemas -j 1`

## Failure interpretation
- boundary failures usually mean ownership drift, public-surface creep, or cross-layer coupling
- contract failures usually mean a user-visible artifact or schema changed
- determinism failures mean fixture or serialization output stopped being stable
- schema failures mean SQLite compatibility drifted
- semantics failures mean ranking or comparison behavior changed
