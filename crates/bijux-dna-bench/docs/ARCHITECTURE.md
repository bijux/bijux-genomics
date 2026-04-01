# Architecture

## Intent
`bijux-dna-bench` owns benchmark orchestration, not workflow planning or execution.

## Crate tree
```text
crates/bijux-dna-bench/
├── bench/          # suite ownership and human-facing benchmark catalog
├── src/
│   ├── artifacts/  # deterministic artifact writers
│   ├── public_api/ # curated stable surface and explicit stable-surface owner
│   ├── repo/       # workspace path discovery, repository contracts, and persisted run artifacts
│   └── workflow/   # suite loading, summarization, evaluation, persistence, and fairness checks
└── tests/
    ├── boundaries/   # source-tree guardrails
    ├── contracts/    # API and contract behavior
    ├── determinism/  # stable ordering and snapshots
    ├── schemas/      # reserved public-surface/schema coverage
    └── semantics/    # gate semantics
```

## Source responsibilities
- `src/lib.rs`: one thin root that re-exports `public_api`
- `src/public_api/`: stable benchmark surface with an explicit stable-surface owner
- `src/workflow/mod.rs`: summarization entrypoint
- `src/workflow/evaluation.rs`: gating and comparison
- `src/workflow/run_suite.rs`: suite persistence and resume orchestration
- `src/workflow/summary_fairness.rs`: fairness and input-consistency checks for summarization
- `src/workflow/summary_scope.rs`: grouping and stratum scopes for summarization
- `src/workflow/summary_statistics.rs`: bootstrap and outlier helper behavior for summaries
- `src/repo/repo_root.rs`: benchmark repository root discovery
- `src/repo/repository.rs`: repository trait contract
- `src/repo/run_metadata.rs`: benchmark run metadata model
- `src/repo/run_artifacts/`: persisted run-artifact loaders by artifact kind
- `src/repo/sqlite/queries/run_index/`: run-index repository queries and metadata-path policy
- `src/repo/workspace_paths.rs`: benchmark workspace path policy
- `src/artifacts/writer.rs`: canonical artifact serialization

## Guardrails
The tree is enforced by `tests/boundaries/architecture_tree.rs` and existing contract checks under
`tests/contracts/`.
