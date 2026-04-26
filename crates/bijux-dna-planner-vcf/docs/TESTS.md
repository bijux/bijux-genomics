# Tests

VCF planner tests protect deterministic stage plans, graph topology, explain payloads, coverage-regime behavior, tool overrides, planner refusals, and boundary rules.

## Entry Points
- `tests/boundaries.rs` — documentation layout, command inventory, architecture, dependency, public API, and source-effect boundary checks.
- `tests/contracts.rs` — snapshot contracts and planner refusal coverage.
- `tests/guardrails.rs` — crate-local guardrail smoke coverage.

## Boundary Modules
- `tests/boundaries/architecture_tree.rs` — locks the intentional crate layout.
- `tests/boundaries/command_inventory.rs` — keeps `docs/COMMANDS.md` aligned with VCF domain stages.
- `tests/boundaries/dependency_graph.rs` — protects allowed runtime and test dependencies.
- `tests/boundaries/docs_layout.rs` — enforces one root README and ten docs under `docs/`.
- `tests/boundaries/public_api_docs.rs` — keeps documented exports compilable.
- `tests/boundaries/source_effects.rs` — prevents process, network, and mutation primitives in production source.

## Contract Coverage
- Default downstream plans for diploid, low-coverage GL, and pseudohaploid regimes.
- Tool override behavior for diploid downstream planning.
- Requested stage subset planning with panel context.
- Duplicate, unknown, coverage-incompatible, and out-of-order stage refusals.
- eDNA and pollen domain refusals.
- Stage parameter override validation.

## Snapshots
Snapshot files live under `tests/snapshots/`. Snapshot changes require review of stage order, graph edges, command specs, reference context, panel locks, or explain contract intent.

## Standard Command
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-vcf --no-default-features
```
