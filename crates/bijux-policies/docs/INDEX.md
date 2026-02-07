# bijux-policies Docs Index

## Scope
Defines the policy suite that governs architecture, purity, and documentation placement.

## Effects
No runtime effects; policies only inspect source/layout.

## Boundaries
Policies enforce boundaries across crates and prevent drift.

## Extension Points
Add new policies in `tests/surface` or `tests/deps` and document them here.

## How to Test
See `TESTS.md` for test mapping and fixtures.

## Policy modules
### surface
- `docs_required_policy.rs`
- `docs_spine.rs`
- `docs_tree_contract.rs`
- `docs_index_quality.rs`
- `domain_purity.rs`
- `id_literal_policy.rs`
- `mod_naming_policy.rs`
- `no_duplicate_canonicalizers.rs`
- `no_empty_dirs_policy.rs`
- `no_helpers_policy.rs`
- `no_policy_duplication.rs`
- `no_serde_json_writer.rs`
- `no_src_crowd_policy.rs`
- `no_thin_modules_policy.rs`
- `planner_purity.rs`
- `policy_docs_anchor.rs`
- `readme_policy.rs`
- `runner_tree_policy.rs`
- `ssot_catalog_authority.rs`
- `stage_specs_purity.rs`
- `style_policy.rs`
- `tool_id_uniqueness.rs`
### deps
- `dependency_budgets.rs`
- `dependency_boundaries.rs`
- `effect_boundary_map.rs`
- `infra_boundaries.rs`
- `qa_dependency_policy.rs`
### data
- `contract_handshake.rs`
### tooling
- `docs_links.rs`
- `makefile_policies.rs`
- `policies.rs`
