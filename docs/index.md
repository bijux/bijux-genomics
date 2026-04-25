# Bijux Genomics Docs

Welcome to the Bijux Genomics documentation. Start with the overview and then dive into architecture, science, operations, policies, and reference material.

<!-- bijux-genomics-badges:generated:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-genomics/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-genomics/actions/workflows/ci.yml/badge.svg)](https://github.com/bijux/bijux-genomics/actions/workflows/ci.yml)
[![deploy-docs](https://img.shields.io/badge/deploy--docs-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/actions)
[![release-crates](https://img.shields.io/badge/release--crates-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/actions)
[![release-pypi](https://img.shields.io/badge/release--pypi-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/actions)
[![release-ghcr](https://img.shields.io/badge/release--ghcr-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/actions)
[![release-github](https://img.shields.io/badge/release--github-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/actions)
[![release](https://img.shields.io/badge/release-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/releases)
[![ghcr](https://img.shields.io/badge/ghcr-no%20status-9CA3AF)](https://github.com/bijux?tab=packages&repo_name=bijux-genomics)
[![published packages](https://img.shields.io/badge/published%20packages-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics)

[![Repository docs](https://img.shields.io/badge/docs-no%20status-9CA3AF?logo=materialformkdocs&logoColor=white)](https://github.com/bijux/bijux-genomics/tree/main/docs)
<!-- bijux-genomics-badges:generated:end -->

## Getting started
- [What is Bijux Genomics](00-intro/WHAT_IS_BIJUX.md)
- [Scope](00-intro/SCOPE.md)
- [Quickstart](00-intro/QUICKSTART.md)

## Core map
- [Architecture Overview](10-architecture/ARCHITECTURE_OVERVIEW.md)
- [Boundary Map](10-architecture/BOUNDARY_MAP.md)
- [SSOT](10-architecture/SSOT.md)
- [Contract Spine](10-architecture/CONTRACT_SPINE.md)
- [Configs Index](../configs/index.md)

## Operations
- [Operations Index](30-operations/index.md)

## Policy checks
- [CLI conventions and snapshot](cli/index.md)
- `api_boundaries.rs`
- `architecture_pointer_policy.rs`
- `benchmark_dependency_policy.rs`
- `boundary_enforcement.rs`
- `cli_dependency_policy.rs`
- `core_layering.rs`
- `core_purity.rs`
- `coverage_cfg_policy.rs`
- `deep_imports.rs`
- `dependency_boundaries.rs`
- `dependency_budgets.rs`
- `dependency_graph.rs`
- `dependency_graph_contracts.rs`
- `dev_deps_policy.rs`
- `docs_index_quality.rs`
- `docs_placement_policy.rs`
- `docs_required_policy.rs`
- `docs_spine.rs`
- `docs_spine_contract.rs`
- `domain_dependency_policy.rs`
- `domain_purity.rs`
- `effect_boundary_map.rs`
- `empty_tests_dirs.rs`
- `fixtures_policy.rs`
- `guardrails.rs`
- `hashmap_policy.rs`
- `heavy_deps_policy.rs`
- `id_literal_policy.rs`
- `ignored_tests_policy.rs`
- `infra_boundaries.rs`
- `mod_naming_policy.rs`
- `no_cfg_coverage.rs`
- `no_duplicate_canonicalizers.rs`
- `no_duplicate_policy_checks.rs`
- `no_empty_dirs_policy.rs`
- `no_helpers_policy.rs`
- `no_policy_duplication.rs`
- `no_repo_tree_snapshots.rs`
- `no_serde_json_writer.rs`
- `no_standalone_bijux_cli.rs`
- `no_thin_modules_policy.rs`
- `ownership_contract.rs`
- `path_policies.rs`
- `pipelines_dependency_policy.rs`
- `planner_purity.rs`
- `policy_docs_anchor.rs`
- `purity_scans.rs`
- `qa_dependency_policy.rs`
- `readme_policy.rs`
- `runner_tree_policy.rs`
- `snapshot_policy.rs`
- `ssot_catalog_authority.rs`
- `stage_specs_purity.rs`
- `style_policy.rs`
- `surface_safety_policies.rs`
- `tests_taxonomy_policy.rs`
- `tool_id_uniqueness.rs`
- `workspace.rs`
- `workspace_paths.rs`
- `layout_contracts.rs`
