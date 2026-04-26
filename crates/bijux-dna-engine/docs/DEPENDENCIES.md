# Dependencies

`bijux-dna-engine` sits between planned core contracts and runtime runner
interfaces. Its dependency graph must stay small enough that orchestration does
not absorb planning, domain semantics, or backend execution.

## Normal Dependencies

- `anyhow` - error propagation for orchestration paths.
- `bijux-dna-core` - execution graph, artifact, run record, identifier, metrics,
  and canonical contract types.
- `bijux-dna-runtime` - runner interface, invocation/result types, run layout,
  and canonical recording helpers.
- `bijux-dna-infra` - filesystem helpers for engine-owned recording and contract
  verification.
- `serde` and `serde_json` - public config/event serialization and JSON contract
  parsing.
- `tracing` - engine-owned execution and artifact verification events.
- `thiserror` - `EngineError` taxonomy.
- `chrono` - execution record timestamps.

## Dev Dependencies

- `bijux-dna-policies` - shared guardrail tests.
- `cargo_metadata` - dependency graph assertions.
- `tempfile` - isolated run-layout fixtures.
- `walkdir` - effect-boundary and manifest-layout scans.

## Forbidden Edges

Normal dependencies must not include planners, domains, stage crates,
`bijux-dna-runner`, `bijux-dna-environment`, CLI crates, benchmark crates, or API
adapters. Those layers may call the engine, but the engine must not call back
into them.

## Enforcement

- `tests/contracts/architecture.rs` rejects `bijux-dna-runner`.
- `tests/boundaries/effect_boundary.rs` rejects process/container effects.
- Workspace dependency policy rejects domain, stage, planner, runner, and
  environment edges.
