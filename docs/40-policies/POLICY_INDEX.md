# POLICY_INDEX

## What
Canonical index of all policies enforced by `bijux-dna-policies`.

## Why
Makes enforcement transparent and searchable.

## Non-goals
- Redefining policy logic outside `bijux-dna-policies`.

## Contracts
Policies are enforced by tests under
[crates/bijux-dna-policies/tests/](../../crates/bijux-dna-policies/tests/) and categorized in
[POLICY_MATRIX.md](POLICY_MATRIX.md).

## Examples
- Look up a policy ID here, then jump to the test file for implementation details.

## Failure modes
- Missing entries create invisible enforcement gaps.

## Policy Table
| Policy ID | Purpose | Prevents | Applies to | Test file |
| --- | --- | --- | --- | --- |
| dependency_budgets | keep deps lean | heavy deps creep | all crates | `tests/deps/dependency_budgets.rs` |
| dependency_boundaries | enforce layering | forbidden edges | all crates | `tests/deps/dependency_boundaries.rs` |
| dependency_graph | forbid edges | runner↔engine coupling | all crates | `tests/deps/dependency_graph.rs` |
| effect_boundary_map | enforce effect boundaries | process spawn outside runner | all crates | `tests/deps/effect_boundary_map.rs` |
| infra_boundaries | infra purity | domain semantics in infra | infra | `tests/deps/infra_boundaries.rs` |
| qa_dependency_policy | no prod → qa deps | qa leakage | all crates | `tests/deps/qa_dependency_policy.rs` |
| docs_required_policy | docs presence | missing docs | all crates | `tests/surface/docs_required_policy.rs` |
| docs_spine | docs placement | docs in src | all crates | `tests/surface/docs_spine.rs` |
| docs_tree_contract | docs drift | untracked doc changes | root docs | `tests/surface/docs_tree_contract.rs` |
| docs_index_quality | index quality | missing sections/links | all crates | `tests/surface/docs_index_quality.rs` |
| domain_purity | domain purity | execution logic in domain | domain crates | `tests/surface/domain_purity.rs` |
| id_literal_policy | SSOT IDs | string IDs | all crates | `tests/surface/id_literal_policy.rs` |
| mod_naming_policy | naming hygiene | legacy/internal misuse | all crates | `tests/surface/mod_naming_policy.rs` |
| no_duplicate_canonicalizers | single canonicalization | duplicate hash/paths | all crates | `tests/surface/no_duplicate_canonicalizers.rs` |
| no_empty_dirs_policy | no empty dirs | marker-only dirs | all crates | `tests/surface/no_empty_dirs_policy.rs` |
| no_helpers_policy | no helpers buckets | helpers/utils | all crates | `tests/surface/no_helpers_policy.rs` |
| no_policy_duplication | centralize policy logic | policy redefs | all crates | `tests/surface/no_policy_duplication.rs` |
| no_serde_json_writer | canonical JSON only | raw serde_json writes | all crates | `tests/surface/no_serde_json_writer.rs` |
| no_thin_modules_policy | no thin dirs | mod.rs-only dirs | all crates | `tests/surface/no_thin_modules_policy.rs` |
| planner_purity | planner purity | parsing logic in planners | planners | `tests/surface/planner_purity.rs` |
| policy_docs_anchor | policy docs link | missing policy listing | policies crate | `tests/surface/policy_docs_anchor.rs` |
| readme_policy | README freshness | missing sections/links | all crates | `tests/surface/readme_policy.rs` |
| runner_tree_policy | runner structure | drifting tree | runner | `tests/surface/runner_tree_policy.rs` |
| ssot_catalog_authority | single owner IDs | duplicate IDs | all crates | `tests/surface/ssot_catalog_authority.rs` |
| stage_specs_purity | stage purity | command building in stages | stages | `tests/surface/stage_specs_purity.rs` |
| style_policy | style rules | doc/style drift | all crates | `tests/surface/style_policy.rs` |
| tool_id_uniqueness | tool IDs unique | duplicate tool IDs | planners | `tests/surface/tool_id_uniqueness.rs` |
| contract_handshake | plan/manifest/report compatibility | mismatch schemas | all crates | `tests/data/contract_handshake.rs` |
| docs_links | link integrity | broken links | root docs | `tests/tooling/docs_links.rs` |
| makefile_policies | make targets | missing make rules | root | `tests/tooling/makefile_policies.rs` |
| policies | policy entrypoint | missing registration | policies | `tests/tooling/policies.rs` |
