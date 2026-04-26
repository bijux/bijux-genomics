# Tests

Planner tests protect deterministic plan output, graph topology, command specs, explain payloads, and boundary rules.

## Entry Points
- `tests/boundaries.rs` — shared guardrails and planner boundary checks.
- `tests/contracts.rs` — plan, graph, explain, command, and fixture contracts.
- `tests/determinism.rs` — deterministic plan ordering and stable graph behavior.
- `tests/guardrails.rs` — crate-local guardrail smoke coverage.

## Boundary Modules
- `tests/boundaries/architecture_tree.rs` — crate source, docs, and test tree contract.
- `tests/boundaries/command_inventory.rs` — `docs/COMMANDS.md` command SSOT contract.
- `tests/boundaries/dependency_graph.rs` — allowed runtime and test dependency graph.
- `tests/boundaries/docs_layout.rs` — root `README.md` plus exactly ten `docs/` files.
- `tests/boundaries/public_api_docs.rs` — `docs/PUBLIC_API.md` export list.

## Contract Modules
- `tests/contracts/graph/graph_snapshots.rs` — execution graph snapshots.
- `tests/contracts/graph/docs_graph_snapshots.rs` — docs anchor coverage for graph contracts.
- `tests/contracts/plan/plan_json.rs` — plan JSON schema and command payload behavior.
- `tests/contracts/plan/plan_snapshots.rs` — stage and BAM plan snapshots.
- `tests/contracts/plan/plan_integration.rs` — integration wiring.
- `tests/contracts/plan/pipeline_plan_snapshot.rs` — pipeline helper snapshots.
- `tests/contracts/plan/artifacts_contract.rs` — stage artifact contracts.
- `tests/contracts/plan/params_complete.rs` — params coverage.
- `tests/contracts/plan/contract_handshake.rs` — stage contract handoff.
- `tests/contracts/plan/no_parsing_execution.rs` — planner purity.
- `tests/contracts/explain/explainability.rs` — explain payload contract.

## Fixtures and Snapshots
- Fixtures live under `tests/fixtures/`.
- Snapshots live under `tests/snapshots/`.
- Snapshot changes require review of plan, graph, command, or explain contract intent.

## Standard Command
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-bam --no-default-features
```
