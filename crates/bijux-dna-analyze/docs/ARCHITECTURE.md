# Architecture

## Intent
`bijux-dna-analyze` turns completed runtime artifacts into deterministic analysis outputs. The tree
is organized by enduring responsibility, not by temporary implementation steps.

## Root Layout
```text
crates/bijux-dna-analyze/
├── Cargo.toml
├── README.md
├── docs/
├── src/
└── tests/
```

The crate root intentionally contains only `Cargo.toml`, `README.md`, `docs/`, `src/`, and
`tests/`. All other markdown documentation belongs in `docs/`.

## Docs Layout
The docs allowance is exactly 10 files:

- `ARCHITECTURE.md`: source and test tree map
- `BOUNDARY.md`: allowed ownership, effects, and dependencies
- `CHANGE_RULES.md`: compatibility and review rules
- `COMMANDS.md`: command and mode source of truth
- `DECISIONS.md`: comparison, ranking, and explainability semantics
- `DETERMINISM.md`: stable-output contract
- `FAILURE_HANDLING.md`: failure classes, hints, and diagnosis paths
- `PUBLIC_API.md`: curated stable API surface
- `REPORT_CONTRACT.md`: report bundle, schema, privacy, and performance contract
- `TESTS.md`: test-suite map and fixture guidance

## Source Layout
- `src/lib.rs`: crate entrypoint; delegates to the pipeline and re-exports `public_api`
- `src/aggregate/`: metrics aggregation, metric schemas, and report-ready metric facts
- `src/api/`: typed request, response, render-option, and metric-id models
- `src/contracts/`: versioned analyze handshake
- `src/decision/`: comparison, scoring, tie handling, weights, and decision traces
- `src/diagnostics/`: durable error types for load and aggregate flows
- `src/exports/`: dashboard facts, run summaries, stage summaries, and export support helpers
- `src/failure/`: failure classification and structured remediation hints
- `src/load/`: runtime artifact loading, run indexes, summaries, and optional SQLite/parquet readers
- `src/model/`: typed analysis records and JSON wrappers
- `src/pipeline/`: orchestration plus load, validate, compute, report, and render steps
- `src/public_api/`: curated stable re-export namespaces
- `src/report/`: report construction, benchmark views, sections, render model, and renderers
- `src/semantics/`: metric semantics and missing-data policy

## Test Layout
- `tests/boundaries.rs`: source tree, docs layout, public surface, ownership, and layering
- `tests/contracts.rs`: report, facts, metrics, pipeline, dashboard, loader, and API contracts
- `tests/determinism.rs`: stable fixture and serialization checks
- `tests/schemas.rs`: SQLite migration, latest-schema, query lint, and deterministic query checks
- `tests/semantics.rs`: ranking, comparison, selection, tie, and decision-trace behavior
- `tests/fixtures/`: durable input and generated artifact fixtures
- `tests/snapshots/`: blessed contract outputs

## Dependency direction
- `load/` reads and validates produced artifacts; it must not call decision or report code.
- `decision/` computes comparisons and rankings; it must not perform IO or call load/report code.
- `report/` builds and renders report models from typed inputs; it must not query facts directly
  through `load/`.
- `pipeline/` is the only namespace that may coordinate load, decision, and report concerns.
- `public_api/` curates stable exports; `lib.rs` stays intentionally thin.
- benchmark execution crates are test-only dependencies and must not be imported from `src/`.

## Guardrails
The source and documentation tree are enforced by:

- `tests/boundaries/architecture_tree.rs`
- `tests/boundaries/docs_layout.rs`
- `tests/boundaries/guardrails.rs`
