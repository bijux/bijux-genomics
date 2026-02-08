# Tests

## What
Maps policy tests to intent and failure meaning.

## Why
Tests become documentation for governance rules.

## Contracts
- Every policy test file must be listed and explained.
- Test names follow `policy__<suite>__<file>__<rule>` for stable search.

## Root
- `guardrails.rs` — runs guardrail checks with per-crate configs.
- `policy_snapshot.rs` — snapshots default guardrail configs.
- `workspace.rs` — repository policy harness.

## data
- `contract_handshake.rs` — planner ⇄ runtime ⇄ analyze handshake fixtures.
- `defaults_policy.rs` — defaults live only in pipelines.
- `policy_snapshot.rs` — serialize guardrail defaults deterministically.
- `purity_scans.rs` — scans for forbidden data-layer tokens.
- `test_data_policies.rs` — test data size + fixture hygiene.

## deps
- `benchmark_dependency_policy.rs` — benchmark crate dependency limits.
- `cli_dependency_policy.rs` — CLI depends only on API + CLI support.
- `core_layering.rs` — core layering rules.
- `dependency_boundaries.rs` — cross-crate dependency boundaries.
- `dependency_budgets.rs` — dependency size caps.
- `dependency_graph.rs` — boundary map matches dependency DAG.
- `dev_deps_policy.rs` — dev dependency allowlist.
- `domain_dependency_policy.rs` — domain crates use only pure deps.
- `effect_boundary_map.rs` — effect boundary map enforcement.
- `heavy_deps_policy.rs` — heavy deps must be feature-gated.
- `infra_boundaries.rs` — infra has no domain semantics.
- `pipelines_dependency_policy.rs` — pipelines avoid stages/planners.
- `qa_dependency_policy.rs` — prod crates do not depend on QA crates.

## surface
- `api_boundaries.rs` — API/CLI avoid internal planner/engine imports.
- `architecture_pointer_policy.rs` — architecture docs are brief pointers.
- `core_purity.rs` — core avoids runtime/system deps.
- `crate_tree_contract.rs` — snapshot crate tree layout.
- `deep_imports.rs` — disallow deep internal imports.
- `docs_index_quality.rs` — docs index required sections + links.
- `docs_required_policy.rs` — required crate docs exist + naming rules.
- `docs_spine.rs` — root docs spine + metadata headers.
- `docs_spine_contract.rs` — snapshot crate docs spine.
- `docs_tree_contract.rs` — snapshot docs tree structure.
- `domain_purity.rs` — domain crates stay pure.
- `guardrails.rs` — guardrails for bijux-policies.
- `id_literal_policy.rs` — ID literals only in canonical owners.
- `mod_naming_policy.rs` — module naming rules.
- `no_duplicate_canonicalizers.rs` — canonicalizers only in core.
- `no_empty_dirs_policy.rs` — no empty or placeholder dirs.
- `no_helpers_policy.rs` — ban helpers/utils buckets.
- `no_policy_duplication.rs` — policies only in bijux-policies.
- `no_serde_json_writer.rs` — canonical JSON writer only via core.
- `no_src_crowd_policy.rs` — limit root src modules.
- `no_thin_modules_policy.rs` — no thin module directories.
- `ownership_contract.rs` — ownership rules for IDs/defaults/metrics.
- `path_policies.rs` — path and write location rules.
- `planner_purity.rs` — planners are declarative only.
- `readme_policy.rs` — README required sections + links.
- `runner_tree_policy.rs` — runner tree layout contract.
- `ssot_catalog_authority.rs` — catalog SSOT ownership.
- `stage_specs_purity.rs` — stage specs are declarative.
- `style_policy.rs` — style policy entrypoint.
- `test_grouping_policy.rs` — tests grouped into suites.
- `tool_id_uniqueness.rs` — tool IDs unique across planners.
- `workspace.rs` — workspace layout + guardrail contracts.

## tooling
- `ci_tools_policy.rs` — CI uses make-only commands + YAML scoping.
- `command_spawn_policy.rs` — command spawning confined to runner/env.
- `docs_links.rs` — docs links are resolvable.
- `makefile_policies.rs` — root makefile is single source.
- `no_appledouble.rs` — ban AppleDouble/.DS_Store artifacts.
- `policies.rs` — cross-cutting policy checks.
