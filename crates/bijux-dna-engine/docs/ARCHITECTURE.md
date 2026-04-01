# bijux-dna-engine Architecture

## Intent
`bijux-dna-engine` executes a fully formed graph. It does not plan workflows or perform backend
effects directly.

## Crate tree
```text
crates/bijux-dna-engine/
├── src/
│   ├── control.rs         # cancellation control
│   ├── engine_config.rs   # engine execution policy
│   ├── engine_driver.rs   # Engine entrypoint
│   ├── errors.rs          # engine-owned error taxonomy
│   ├── executor/          # execution orchestration internals
│   ├── observability.rs   # events and hooks
│   └── public_api/        # curated stable surface
└── tests/
    ├── boundaries/        # architecture and effect-boundary guardrails
    ├── contracts/         # execution and recording contracts
    ├── determinism/       # replay and manifest stability
    └── support/           # reusable test helpers
```

## Executor tree
```text
src/executor/
├── contracts/     # output, metrics, and run-artifact verification
├── graph.rs       # graph normalization and ordering preparation
├── mod.rs         # executor facade
├── recording/     # execution-record payload and persistence
├── step_execution.rs
└── topology.rs
```

## Dependency direction
- `engine_driver.rs` applies `EngineConfig` and delegates execution to `executor/`
- `executor/graph.rs` prepares ordered steps from the normalized `ExecutionGraph`
- `executor/step_execution.rs` is the only place that coordinates runner calls, recording, and
  contract verification
- `public_api/` curates the surface; `lib.rs` stays intentionally thin

## Guardrails
The source and test tree are enforced by `tests/boundaries/architecture_tree.rs` and
`tests/boundaries/effect_boundary.rs`.
