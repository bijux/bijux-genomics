# Tests

Planner tests protect deterministic plan output, graph topology, command specs, explain payloads, benchmark fan-out, and boundary rules.

## Entry Points
- `tests/boundaries.rs` — shared guardrails and planner boundary checks.
- `tests/contracts.rs` — plan, graph, explain, command, and fixture contracts.
- `tests/determinism.rs` — deterministic plan ordering and stable graph behavior.
- `tests/guardrails.rs` — crate-local guardrail smoke coverage.

## Boundary Modules
- `tests/boundaries/architecture_tree.rs` — locks the intentional crate layout.
- `tests/boundaries/command_inventory.rs` — keeps `docs/COMMANDS.md` aligned with stage authorities.
- `tests/boundaries/dependency_graph.rs` — protects allowed runtime and test dependencies.
- `tests/boundaries/docs_layout.rs` — enforces one root README and ten docs under `docs/`.
- `tests/boundaries/public_api_docs.rs` — keeps documented exports compilable.
- `tests/boundaries/source_effects.rs` — prevents process, network, and mutation primitives in production source.

## Contract Modules
- `tests/contracts/benchmark_fanout.rs` — benchmark fan-out graph behavior.
- `tests/contracts/benchmark_profiles.rs` — benchmark profile governance.
- `tests/contracts/docs.rs` — docs anchor and registry publication contracts.
- `tests/contracts/explain/` — explain payload shape and docs anchors.
- `tests/contracts/graph/` — graph topology snapshots.
- `tests/contracts/plan/` — stage plan JSON, params, artifacts, graph policy, and purity.
- `tests/contracts/preprocess_contract.rs` — preprocess policy handoff.
- `tests/contracts/stage_instance_ids.rs` — stage instance identity.
- `tests/contracts/tool_maturity.rs` — stage-tool maturity surface.
- `tests/contracts/tool_selection.rs` — default and allowed tool selection.
- `tests/contracts/toolset_*` — toolset selection, modes, and overrides.

## Fixtures and Snapshots
- Fixtures live under `tests/fixtures/`.
- Snapshot outputs live under `tests/snapshots/`.
- Snapshot changes require review of plan, graph, command, benchmark, or explain contract intent.

## Standard Command
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-fastq --no-default-features
```
