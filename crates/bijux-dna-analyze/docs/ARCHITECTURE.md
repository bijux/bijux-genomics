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

## Evidence bundle ownership
`src/exports/evidence_bundle.rs` retains the stable evidence export surface through private
responsibility owners:

- `evidence_bundle/model.rs`: versioned evidence, profile, comparison, and reviewer records
- `evidence_bundle/construction.rs`: completed-run acquisition and evidence projection construction
- `evidence_bundle/verification.rs`: typed profile validation and live evidence/file verification
- `evidence_bundle/publication.rs`: evidence, methods, and profile composition and publication
- `evidence_bundle/reviewer_challenges.rs`: reference validation and the clocked reviewer journal
- `evidence_bundle/comparison.rs`: bundle acquisition and deterministic evidence comparison

The records do not depend on acquisition or publication. Construction, verification, comparison,
and the reviewer journal depend on those records. Publication composes construction and verification;
it does not coordinate workflow execution or scheduler state. Typed profile validation remains free
of acquisition, while file verification and publication keep their existing effects and order.
`exports/mod.rs`, the public API, and API/CLI callers keep their existing exports. Existing evidence
contracts remain registered once in `tests/contracts.rs`; see `docs/TESTS.md` for the suite map.

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
