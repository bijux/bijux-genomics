# Architecture

## Intent
`bijux-dna-analyze` turns completed runtime artifacts into deterministic analysis outputs. The tree
is organized by enduring responsibility, not by temporary implementation steps.

## Crate tree
```text
crates/bijux-dna-analyze/
├── src/
│   ├── aggregate/      # metrics aggregation and report-ready facts
│   ├── api/            # typed request and response models
│   ├── contracts/      # versioned analysis handshake
│   ├── decision/       # comparison, scoring, and ranking logic
│   ├── diagnostics/    # load and aggregate error taxonomies
│   ├── exports/        # summary and dashboard artifact writers
│   ├── failure/        # failure classification and hint generation
│   ├── load/           # runtime artifact and SQLite-backed loading
│   ├── model/          # shared analysis data models
│   ├── pipeline/       # pipeline orchestration and step entrypoints
│   ├── public_api/     # curated stable surface
│   ├── report/         # report builders, sections, and renderers
│   └── semantics/      # metric interpretation policy
└── tests/
    ├── boundaries/     # architecture and public-surface guardrails
    ├── contracts/      # behavior and artifact contracts
    ├── determinism/    # stable fixture invariants
    ├── schemas/        # SQLite schema compatibility
    └── semantics/      # ranking and comparison semantics
```

## Source responsibilities
- `src/lib.rs`: one small crate root that delegates to the pipeline and re-exports `public_api`
- `src/pipeline/steps/`: canonical load, validate, compute, report, and render stages
- `src/exports/`: writes derived artifacts without owning report construction
  through `facts_summary.rs` and `facts_support/`
- `src/report/render_model/`: renderer-owned report model contracts and construction policy
- `src/report/build/report_sections/`: report builder-owned section assembly helpers split by
  durable output concern
- `src/diagnostics/`: durable error types shared across internal namespaces

## Dependency direction
- `load/`, `decision/`, and `report/` do not call each other directly outside the pipeline.
- `pipeline/` is the only namespace that may coordinate load, decision, and report concerns
  together.
- `public_api/` curates stable exports; `lib.rs` stays intentionally thin.

## Guardrails
The source tree is enforced by `tests/boundaries/guardrails.rs` and
`tests/boundaries/architecture_tree.rs`.
