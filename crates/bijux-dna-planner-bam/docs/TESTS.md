# Tests

Planner tests protect deterministic plan output, graph topology, command specs, explain payloads, and boundary rules.

## Entry Points
- `tests/boundaries.rs` — shared guardrails and planner boundary checks.
- `tests/contracts.rs` — plan, graph, explain, command, and fixture contracts.
- `tests/determinism.rs` — deterministic plan ordering and stable graph behavior.

## Boundary Modules
- `tests/boundaries/architecture_tree.rs` — crate source, docs, and test tree contract.
- `tests/boundaries/command_inventory.rs` — `docs/COMMANDS.md` command SSOT contract.
- `tests/boundaries/dependency_graph.rs` — allowed runtime and test dependency graph.
- `tests/boundaries/docs_layout.rs` — root `README.md` plus exactly ten `docs/` files.
- `tests/boundaries/public_api_docs.rs` — `docs/PUBLIC_API.md` export list.
- `tests/boundaries/source_effects.rs` — forbidden source-side execution, network, and mutation APIs.

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
- `tests/contracts/plan/registry_root.rs` — private registry primary choice,
  refreshed observations, explicit root admission, and missing, malformed, or
  unreadable provider refusal. Independent child processes keep environment and
  filesystem state private under repository artifacts.
- `tests/contracts/explain/explainability.rs` — explain payload contract.

## Fixtures and Snapshots
- Fixtures live under `tests/fixtures/`.
- Snapshots live under `tests/snapshots/`.
- Snapshot changes require review of plan, graph, command, or explain contract intent.

## Standard Command
Run from the repository root with Python 3 available. The report-language contracts
require `PYTHON` to name an absolute interpreter path.

```bash
PYTHON="$(python3 -c 'import sys; print(sys.executable)')" \
CARGO_TARGET_DIR=artifacts/cargo-target \
cargo test -p bijux-dna-planner-bam --no-default-features
```

Shared guardrail parser regressions are owned and executed by the policy crate. This boundary suite retains its crate-specific architecture checks; it does not compile another crate's test source. Constructor-only guardrail wrappers provide no source-check coverage and are retired.

`tests/contracts/plan/samtools_report_language.rs` executes the exact Python report heredocs emitted by public samtools filter, markdup and endogenous-content argument producers. Genuine interpreter success/refusal and existing mapping-summary body controls precede boolean/optional-digest observations. It does not execute samtools, validate alignments, or certify backend science.
