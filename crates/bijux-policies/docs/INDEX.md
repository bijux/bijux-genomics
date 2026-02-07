# Policy Index

## What
Defines the policy suite that governs architecture, purity, and documentation placement.

## Why
A single index prevents policy sprawl and makes CI failures discoverable.

## Non-goals
- Rewriting policy logic outside `bijux-policies`.
- Duplicating checks in other crates.

## Contracts
- Policies live under `crates/bijux-policies/tests/{deps,surface,data,tooling}`.
- Every policy test file must be listed below.

## Examples
- `make policy-fast` runs the fast policy subset.
- `make policy-full` runs the complete suite.

## Failure modes
- Missing policy entries fail `policy_docs_anchor` tests.

## Policy modules
### deps
- `dependency_budgets.rs`
- `effect_boundary_map.rs`
- `infra_boundaries.rs`
- `qa_dependency_policy.rs`

### surface
- `docs_required_policy.rs`
- `docs_spine.rs`
- `docs_tree_contract.rs`
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
- `policy_docs_anchor.rs`
- `planner_purity.rs`
- `runner_tree_policy.rs`
- `ssot_catalog_authority.rs`
- `stage_specs_purity.rs`
- `style_policy.rs`
- `tool_id_uniqueness.rs`

### data
- `contract_handshake.rs`

### tooling
- `docs_links.rs`
