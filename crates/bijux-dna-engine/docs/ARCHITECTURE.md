# bijux-dna-engine Architecture

## Intent
`bijux-dna-engine` executes a fully formed graph. It does not plan workflows or perform backend
effects directly.

## Crate tree
```text
crates/bijux-dna-engine/
├── src/
│   ├── control/           # cancellation token contracts and state transitions
│   ├── engine_config/     # engine execution policy and graph application
│   ├── engine_driver.rs   # Engine entrypoint
│   ├── errors.rs          # engine-owned error taxonomy
│   ├── executor/          # execution orchestration internals
│   ├── observability/     # events and hook contracts
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
├── graph/         # graph normalization and ordering preparation
├── mod.rs         # executor facade
├── recording/     # execution-record payload and persistence
├── step_execution/ # runner lifecycle and execution-record shaping
└── topology.rs
```

## Dependency direction
- `engine_driver.rs` delegates graph policy application to `engine_config/` and execution to
  `executor/`
- `executor/graph/` prepares ordered steps from the normalized `ExecutionGraph`
- `executor/step_execution/` is the only place that coordinates runner calls, recording, and
  contract verification
- `public_api/` curates the surface; `lib.rs` stays intentionally thin

## Guardrails
The source and test tree are enforced by `tests/boundaries/architecture_tree.rs` and
`tests/boundaries/effect_boundary.rs`.
