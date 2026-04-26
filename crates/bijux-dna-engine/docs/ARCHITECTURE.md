# bijux-dna-engine Architecture

## Intent
`bijux-dna-engine` executes a fully formed graph. It does not plan workflows,
select tools, interpret domain semantics, or perform backend effects directly.
The engine is the sequential coordinator between immutable core contracts and a
caller-provided runtime `Runner`.

## Crate tree
```text
crates/bijux-dna-engine/
├── README.md
├── docs/
│   ├── ARCHITECTURE.md
│   ├── BOUNDARY.md
│   ├── CHANGE_RULES.md
│   ├── COMMANDS.md
│   ├── DEPENDENCIES.md
│   ├── DETERMINISM.md
│   ├── EFFECTS.md
│   ├── INDEX.md
│   ├── PUBLIC_API.md
│   └── TESTS.md
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
├── facade.rs      # public executor entrypoint
├── graph/         # graph normalization and ordering preparation
├── mod.rs         # module declarations and facade re-export
├── recording/     # execution-record payload and persistence
└── step_execution/ # runner lifecycle and execution-record shaping
```

```text
src/executor/graph/
├── mod.rs         # graph normalization
└── topology.rs    # deterministic topological ordering
```

## Execution Flow

```text
Engine::execute
  ├── EngineConfig::validate
  ├── apply_engine_config
  ├── executor::graph::normalize_for_execution
  └── executor::step_execution::execute_ordered_steps
      ├── cancellation check
      ├── EngineEvent::StepStart
      ├── Runner::run
      ├── recording::record_execution
      ├── contract verification
      ├── retry or failure handling
      └── EngineEvent::StepEnd
```

## Dependency direction

- `engine_driver.rs` delegates graph policy application to `engine_config/` and
  execution to `executor/`.
- `engine_config/` can depend on core contracts but not executor internals.
- `executor/graph/` prepares ordered steps from the normalized `ExecutionGraph`.
- `executor/step_execution/` is the only place that coordinates runner calls,
  recording, cancellation, retries, timeout checks, and contract verification.
- `executor/contracts/` reads declared outputs and run artifacts but does not
  know how tools produced them.
- `executor/recording/` writes only engine-owned execution records.
- `observability/` defines event and hook contracts; it must not call executor
  internals.
- `public_api/` curates the surface; `lib.rs` stays intentionally thin.

## Naming Rules

- Use `ExecutionGraph`, `ExecutionStep`, and `Runner` vocabulary when describing
  graph execution. Do not introduce stage-planning vocabulary in engine code.
- Use `run_artifacts` only for engine/runtime truth files under a step output
  directory.
- Keep test helpers purpose-named under `tests/support/`; do not add generic
  helper modules.

## Guardrails

The source, docs, and test tree are enforced by
`tests/boundaries/architecture_tree.rs`. Direct execution effects are enforced
by `tests/boundaries/effect_boundary.rs`.
