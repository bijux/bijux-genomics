# POLICY_MATRIX

This file maps each policy to its purpose and the affected crates.

## Style

- `docs_required_policy.rs` — enforce docs spine (`SCOPE.md`, `ARCHITECTURE.md`, optional `README.md`). All crates.
- `no_thin_modules_policy.rs` — avoid single-file directories unless justified. All crates.
- `no_helpers_policy.rs` — ban helpers/utils/misc buckets without justification. All crates.
- `no_src_crowd_policy.rs` — prevent overcrowded `src/` roots. All crates.
- `mod_naming_policy.rs` — naming conventions (`internal/`, legacy). All crates.
- `style_policy.rs` — entrypoint and docs spine linkage. All crates.

## Boundaries

- `api_boundaries.rs` — API boundary enforcement. bijux-api + dependents.
- `core_purity.rs` — core purity checks. bijux-core.
- `domain_purity.rs` — domain purity checks. bijux-domain-*.
- `planner_purity.rs` — planner purity checks. bijux-planner-*.
- `runner_tree_policy.rs` — runner layout boundaries. bijux-runner.
- `stage_specs_purity.rs` — stage specs remain declarative. bijux-stages-*.

## Dependency

- `benchmark_dependency_policy.rs` — benchmark must not depend on stages/planners. bijux-benchmark.
- `cli_dependency_policy.rs` — CLI depends only on API + CLI support crates. bijux-cli.
- `pipelines_dependency_policy.rs` — pipelines must avoid execution/planner deps. bijux-pipelines.
- `dependency_graph.rs` — dependency graph invariants. workspace.

## Contracts

- `crate_tree_contract.rs` — expected top-level crate trees. workspace.
- `ownership_contract.rs` — ownership of SSOT catalogs. workspace.
- `ssot_catalog_authority.rs` — prevent duplicate IDs across catalogs. workspace.
- `tool_id_uniqueness.rs` — tool ID uniqueness. planners/domains.
- `id_literal_policy.rs` — forbid ID string literals. workspace.
- `no_duplicate_canonicalizers.rs` — canonicalization only in core. workspace.
- `no_serde_json_writer.rs` — forbid ad-hoc serde_json::to_writer. workspace.

## Data + Tooling

- `path_policies.rs` — path constraints and test layout. workspace.
- `no_policy_duplication.rs` — policy logic must live here. workspace.
- `workspace.rs` — workspace-level invariants. workspace.
