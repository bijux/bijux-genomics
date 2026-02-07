# Contract Spine

Owner: Architecture
Scope: Contract and boundary authority
Last reviewed: 2026-02-07
Contract version: v1
Applies to crates: bijux-core, bijux-runtime, bijux-engine, bijux-analyze

## What
Canonical list of contract artifacts and their owning crates.

## Why
Eliminates ambiguity about where truth lives.

## Non-goals
- Runtime execution details.

## Contracts
| Artifact | Owner crate | Tests / snapshots |
| --- | --- | --- |
| ExecutionGraph | bijux-core | `crates/bijux-core/tests/execution_graph_validate.rs` |
| ExecutionPlan | bijux-stage-contract | `crates/bijux-stage-contract/tests/schema_snapshots.rs` |
| RunManifest | bijux-runtime | `crates/bijux-runtime/tests/runtime_schema_snapshots.rs` |
| RunRecord | bijux-runtime | `crates/bijux-runtime/tests/runtime_schema_snapshots.rs` |
| ToolInvocation | bijux-core | `crates/bijux-core/tests/execution_plan_contract.rs` |
| StageReport | bijux-stages-* | `crates/bijux-stages-*/tests/observer_snapshots.rs` |
| Report bundle | bijux-analyze | `crates/bijux-analyze/tests/report_contract.rs` |
| Defaults ledger | bijux-pipelines | `crates/bijux-pipelines/tests/defaults_ledger.rs` |

## Examples
Each artifact is serialized canonically and stored under the run layout.

## Failure modes
Missing or mismatched artifacts fail contract enforcement tests.
