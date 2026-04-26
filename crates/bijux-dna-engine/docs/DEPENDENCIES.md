# Dependencies

`bijux-dna-engine` sits between planned core contracts and runtime runner
interfaces. Its dependency graph must stay small enough that orchestration does
not absorb planning, domain semantics, or backend execution.

## Normal Dependencies

These are the only permitted direct runtime edges:

- `anyhow` - error propagation for orchestration paths.
- `bijux-dna-core` - execution graph, artifact, run record, identifier, metrics,
  and canonical contract types.
- `bijux-dna-infra` - filesystem helpers for engine-owned recording and contract
  verification.
- `bijux-dna-runtime` - runner interface, invocation/result types, run layout,
  and canonical recording helpers.
- `chrono` - execution record timestamps.
- `serde` and `serde_json` - public config/event serialization and JSON contract
  parsing.
- `thiserror` - `EngineError` taxonomy.
- `tracing` - engine-owned execution and artifact verification events.

## Dev Dependencies

These are test-only edges and must not move into `[dependencies]`:

- `bijux-dna-policies` - shared guardrail tests.
- `cargo_metadata` - dependency graph assertions.
- `tempfile` - isolated run-layout fixtures.
- `walkdir` - effect-boundary and manifest-layout scans.

## Forbidden Edges

Normal dependencies must not include:

- CLI/API adapters.
- Planners.
- Domain crates.
- Stage crates.
- `bijux-dna-runner`.
- `bijux-dna-environment`.
- Benchmark crates.

Those layers may call the engine, but the engine must not call back into them.
The engine boundary is orchestration over already-planned graphs, not ownership
of planning, domain semantics, backend selection, process spawning, or container
execution.

## Enforcement

- `tests/contracts/architecture.rs` rejects `bijux-dna-runner`.
- `tests/boundaries/dependency_graph.rs` locks the direct normal and dev
  dependency names and rejects planner/domain/stage-runner/environment edges.
- `tests/boundaries/effect_boundary.rs` rejects process/container effects and
  keeps those responsibilities outside this crate.

Run dependency verification from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo tree -p bijux-dna-engine --no-default-features --edges normal,dev
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-engine --test boundaries --no-default-features
```
