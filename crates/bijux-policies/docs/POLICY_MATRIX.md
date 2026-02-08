# POLICY_MATRIX

## What
Authoritative registry of policy tests owned by `bijux-policies`.

## How to use
- Find the test file to locate the rule.
- Use `docs/TESTS.md` for detailed intent and failure meaning.

## Matrix
| Test file | Scope | Intent | Read more |
| --- | --- | --- | --- |
| `tests/guardrails.rs` | root | Run crate guardrails with per-crate configs. | `docs/ENFORCEMENT.md` |
| `tests/policy_snapshot.rs` | root | Snapshot default guardrail configs. | `docs/BLESS_WORKFLOW.md` |
| `tests/workspace.rs` | root | Workspace-level policy harness. | `docs/TESTS.md` |
| `tests/data/contract_handshake.rs` | data | Validate planner ⇄ runtime ⇄ analyze handshake fixtures. | `docs/TESTS.md` |
| `tests/data/defaults_policy.rs` | data | Enforce defaults live only in pipelines. | `docs/TESTS.md` |
| `tests/data/policy_snapshot.rs` | data | Serialize guardrail defaults deterministically. | `docs/BLESS_WORKFLOW.md` |
| `tests/data/purity_scans.rs` | data | Scan for forbidden tokens in data-layer crates. | `docs/TESTS.md` |
| `tests/data/test_data_policies.rs` | data | Ban large binary fixtures and enforce test data rules. | `docs/TESTS.md` |
| `tests/deps/benchmark_dependency_policy.rs` | deps | Benchmark crate dependency limits. | `docs/TESTS.md` |
| `tests/deps/cli_dependency_policy.rs` | deps | CLI depends only on API + CLI support. | `docs/TESTS.md` |
| `tests/deps/core_layering.rs` | deps | Core layering rules (foundation/contract). | `docs/TESTS.md` |
| `tests/deps/dependency_boundaries.rs` | deps | Enforce cross-crate dependency boundaries. | `docs/TESTS.md` |
| `tests/deps/dependency_budgets.rs` | deps | Dependency size caps. | `docs/TESTS.md` |
| `tests/deps/dependency_graph.rs` | deps | Boundary map matches dependency DAG. | `docs/TESTS.md` |
| `tests/deps/dev_deps_policy.rs` | deps | Dev-dependency allowlist. | `docs/TESTS.md` |
| `tests/deps/domain_dependency_policy.rs` | deps | Domain crates use only pure deps. | `docs/TESTS.md` |
| `tests/deps/effect_boundary_map.rs` | deps | Effect boundaries map. | `docs/40-policies/STYLE.md` |
| `tests/deps/heavy_deps_policy.rs` | deps | Heavy deps must be feature-gated. | `docs/TESTS.md` |
| `tests/deps/infra_boundaries.rs` | deps | Infra has no domain semantics. | `docs/TESTS.md` |
| `tests/deps/pipelines_dependency_policy.rs` | deps | Pipelines avoid stages/planners. | `docs/TESTS.md` |
| `tests/deps/qa_dependency_policy.rs` | deps | Prod crates must not depend on QA crates. | `docs/TESTS.md` |
| `tests/surface/api_boundaries.rs` | surface | API/CLI avoid internal planner/engine imports. | `docs/TESTS.md` |
| `tests/surface/architecture_pointer_policy.rs` | surface | Architecture docs are brief pointers. | `docs/TESTS.md` |
| `tests/surface/core_purity.rs` | surface | Core avoids runtime/system deps. | `docs/TESTS.md` |
| `tests/surface/crate_tree_contract.rs` | surface | Snapshot crate tree layout. | `docs/BLESS_WORKFLOW.md` |
| `tests/surface/deep_imports.rs` | surface | Disallow deep internal imports. | `docs/TESTS.md` |
| `tests/surface/docs_index_quality.rs` | surface | Docs index required sections and links. | `docs/TESTS.md` |
| `tests/surface/docs_required_policy.rs` | surface | Required crate docs exist and named correctly. | `docs/TESTS.md` |
| `tests/surface/docs_spine.rs` | surface | Root docs spine + metadata headers. | `docs/TESTS.md` |
| `tests/surface/docs_spine_contract.rs` | surface | Snapshot crate docs spine. | `docs/BLESS_WORKFLOW.md` |
| `tests/surface/docs_tree_contract.rs` | surface | Snapshot docs tree structure. | `docs/BLESS_WORKFLOW.md` |
| `tests/surface/domain_purity.rs` | surface | Domain crates stay pure. | `docs/TESTS.md` |
| `tests/surface/guardrails.rs` | surface | Run guardrails for bijux-policies. | `docs/ENFORCEMENT.md` |
| `tests/surface/id_literal_policy.rs` | surface | ID literals only in canonical owners. | `docs/TESTS.md` |
| `tests/surface/mod_naming_policy.rs` | surface | Module naming rules (no legacy/internal). | `docs/TESTS.md` |
| `tests/surface/no_duplicate_canonicalizers.rs` | surface | Canonicalizers only in core. | `docs/TESTS.md` |
| `tests/surface/no_empty_dirs_policy.rs` | surface | No empty or placeholder dirs. | `docs/TESTS.md` |
| `tests/surface/no_helpers_policy.rs` | surface | Ban helpers/utils buckets. | `docs/TESTS.md` |
| `tests/surface/no_policy_duplication.rs` | surface | Policies only in bijux-policies. | `docs/TESTS.md` |
| `tests/surface/no_serde_json_writer.rs` | surface | Canonical JSON writer only via core. | `docs/TESTS.md` |
| `tests/surface/no_src_crowd_policy.rs` | surface | Limit root src modules. | `docs/TESTS.md` |
| `tests/surface/no_thin_modules_policy.rs` | surface | No thin module directories. | `docs/TESTS.md` |
| `tests/surface/ownership_contract.rs` | surface | Ownership rules for IDs, defaults, metrics. | `docs/TESTS.md` |
| `tests/surface/path_policies.rs` | surface | Path and write location rules. | `docs/TESTS.md` |
| `tests/surface/planner_purity.rs` | surface | Planners are declarative only. | `docs/TESTS.md` |
| `tests/surface/readme_policy.rs` | surface | README required sections + links. | `docs/TESTS.md` |
| `tests/surface/runner_tree_policy.rs` | surface | Runner tree layout contract. | `docs/TESTS.md` |
| `tests/surface/ssot_catalog_authority.rs` | surface | Catalog SSOT ownership. | `docs/TESTS.md` |
| `tests/surface/stage_specs_purity.rs` | surface | Stage specs are declarative. | `docs/TESTS.md` |
| `tests/surface/style_policy.rs` | surface | Style policy entrypoint. | `docs/TESTS.md` |
| `tests/surface/test_grouping_policy.rs` | surface | Tests grouped into suites. | `docs/TESTS.md` |
| `tests/surface/tool_id_uniqueness.rs` | surface | Tool IDs unique across planners. | `docs/TESTS.md` |
| `tests/surface/workspace.rs` | surface | Workspace layout + guardrail contracts. | `docs/TESTS.md` |
| `tests/tooling/ci_tools_policy.rs` | tooling | CI uses make-only commands + YAML scoping. | `docs/TESTS.md` |
| `tests/tooling/command_spawn_policy.rs` | tooling | Command spawning confined to runner/env. | `docs/TESTS.md` |
| `tests/tooling/docs_links.rs` | tooling | Docs links are resolvable. | `docs/TESTS.md` |
| `tests/tooling/makefile_policies.rs` | tooling | Root makefile is single source. | `docs/TESTS.md` |
| `tests/tooling/no_appledouble.rs` | tooling | Ban AppleDouble/.DS_Store artifacts. | `docs/TESTS.md` |
| `tests/tooling/policies.rs` | tooling | Cross-cutting policy checks. | `docs/TESTS.md` |
