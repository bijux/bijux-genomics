# Architecture

## Intent
`bijux-dna-bench` owns benchmark orchestration, not workflow planning or execution.

## Crate tree
```text
crates/bijux-dna-bench/
├── bench/          # suite ownership and human-facing benchmark catalog
├── src/
│   ├── artifacts/  # deterministic artifact writers
│   ├── public_api/ # curated stable surface
│   ├── repo/       # workspace path discovery and persisted run artifacts
│   └── workflow/   # suite loading, summarization, evaluation, and persistence
└── tests/
    ├── boundaries/   # source-tree guardrails
    ├── contracts/    # API and contract behavior
    ├── determinism/  # stable ordering and snapshots
    ├── schemas/      # reserved public-surface/schema coverage
    └── semantics/    # gate semantics
```

## Source responsibilities
- `src/lib.rs`: one thin root that re-exports `public_api`
- `src/public_api/mod.rs`: stable benchmark surface
- `src/workflow/mod.rs`: summarization entrypoint
- `src/workflow/evaluation.rs`: gating and comparison
- `src/workflow/run_suite.rs`: suite persistence and resume orchestration
- `src/repo/workspace_paths.rs`: benchmark workspace path policy
- `src/repo/run_artifacts.rs`: persisted run-artifact loaders
- `src/artifacts/writer.rs`: canonical artifact serialization

## Guardrails
The tree is enforced by `tests/boundaries/architecture_tree.rs` and existing contract checks under
`tests/contracts/`.
