# bijux-dna-engine

## What this crate does
Executes a fully formed `ExecutionGraph` through a `Runner`, records per-step execution truth, and
enforces engine-owned output contracts.

## Boundaries
This crate does not plan workflows, spawn tools directly, or own container/backend execution
adapters. It orchestrates a graph that was already planned elsewhere.

## Execution lifecycle
```text
normalize graph -> order steps -> execute runner invocations -> record execution -> verify artifacts
```

## Internal layout
- `src/public_api/`: curated stable surface for downstream crates
- `src/control.rs`: cancellation control
- `src/observability.rs`: engine events and hook contracts
- `src/engine_config.rs`: engine policy inputs
- `src/engine_driver.rs`: `Engine` construction and execution entrypoint
- `src/executor/`: graph preparation, step orchestration, contract enforcement, topology, and
  execution-record persistence

## Public entrypoints
Start with `PUBLIC_API.md` and `docs/ARCHITECTURE.md`. The crate root is intentionally small and
re-exports the curated API from `src/public_api/mod.rs`.

## Contracts and operating rules
- execution contract: `docs/ENGINE_CONTRACT.md`
- execution model: `docs/ENGINE_MODEL.md`
- effect boundary: `docs/EFFECT_BOUNDARY.md`
- recording truth set: `docs/RECORDING_TRUTH_SET.md`
- change policy: `docs/CHANGE_RULES.md`

## Tests
See `docs/TESTS.md` for the full map. The test tree is organized by enduring intent:
- `tests/boundaries.rs`: source-tree and effect-boundary guardrails
- `tests/contracts.rs`: orchestration, recording, and contract behavior
- `tests/determinism.rs`: stable replay and manifest layout checks
- `tests/support/`: reusable engine integration helpers
