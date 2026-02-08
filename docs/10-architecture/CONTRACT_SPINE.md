# Contract Spine

Owner: Architecture
Scope: Contract and boundary authority
Last reviewed: 2026-02-07
Contract version: v1
Applies to crates: bijux-dna-core, bijux-dna-runtime, bijux-dna-engine, bijux-dna-analyze

## What
Canonical list of contract artifacts and their owning crates.

## Why
Eliminates ambiguity about where truth lives.

## Non-goals
- Runtime execution details.

## Contracts
| Artifact | Owner crate | Tests / snapshots |
| --- | --- | --- |
| ExecutionGraph | bijux-dna-core | `crates/bijux-dna-core/tests/execution_graph_validate.rs` |
| ExecutionPlan | bijux-dna-stage-contract | `crates/bijux-dna-stage-contract/tests/schema_snapshots.rs` |
| RunManifest | bijux-dna-runtime | `crates/bijux-dna-runtime/tests/runtime_schema_snapshots.rs` |
| RunRecord | bijux-dna-runtime | `crates/bijux-dna-runtime/tests/runtime_schema_snapshots.rs` |
| ToolInvocation | bijux-dna-core | `crates/bijux-dna-core/tests/execution_plan_contract.rs` |
| StageReport | bijux-dna-stages-* | `crates/bijux-dna-stages-*/tests/observer_snapshots.rs` |
| Report bundle | bijux-dna-analyze | `crates/bijux-dna-analyze/tests/report_contract.rs` |
| Defaults ledger | bijux-dna-pipelines | `crates/bijux-dna-pipelines/tests/defaults_ledger.rs` |

## Examples
Each artifact is serialized canonically and stored under the run layout.

## Failure modes
Missing or mismatched artifacts fail contract enforcement tests.
