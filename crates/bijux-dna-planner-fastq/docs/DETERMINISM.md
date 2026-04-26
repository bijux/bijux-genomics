# Determinism

The same planner inputs must produce the same stage plan JSON, execution graph topology, explain metadata, and benchmark fan-out shape.

## Deterministic Inputs
- Pipeline ID and pipeline spec.
- Stage bindings and stage toolsets.
- Tool allow/deny lists and explicit overrides.
- FASTQ layout, reference, and bank context.
- Profile defaults and parameter overrides.
- `allow_planned` policy.

## Guarantees
- Stage and route expansion order is stable.
- Graph nodes and edges are ordered before snapshot comparison.
- Explain reasons and defaults diffs are deterministic.
- Contract hashes are recorded in plan details when available.
- Benchmark cohort selection is deterministic for a fixed domain governance surface.

## Enforcement
- `tests/determinism.rs`
- `tests/contracts/graph/graph_snapshots.rs`
- `tests/contracts/plan/plan_snapshots.rs`
- `tests/contracts/explain/explainability.rs`
