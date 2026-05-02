# bijux-dna-engine

`bijux-dna-engine` validates engine policy and a caller-provided run layout,
executes a fully formed `ExecutionGraph` through a caller-provided `Runner`,
records per-step execution truth, and enforces engine-owned output contracts.

This crate follows repository governance documentation. `/Users/bijan/bijux/bijux-g2/bijux-genomics/README.md`,
`README.md`, and `README.md`; re-read
those files before editing this child repository and before committing.

## What this crate does

This crate owns graph execution coordination, engine policy validation,
per-step execution truth recording, and output-contract enforcement for planned
execution graphs.

## Boundaries

This crate does not plan workflows, spawn tools directly, select containers, or
own backend execution adapters. It orchestrates a graph that was already planned
elsewhere.

## Execution lifecycle

```text
normalize graph -> order steps -> execute runner invocations -> record execution -> verify artifacts
```

## Internal layout
- `src/control/`: cancellation token contract and state transitions
- `src/engine_config/`: engine policy inputs and graph-policy application
- `src/engine_driver.rs`: `Engine` construction and execution entrypoint
- `src/executor/`: graph preparation, step orchestration, contract enforcement, topology, and
  execution-record persistence
- `src/observability/`: engine events and hook contracts
- `src/public_api/`: curated stable surface for downstream crates

## Managed Operations

`docs/COMMANDS.md` is the SSOT for callable engine operations:

- `create-engine`
- `execute-graph`
- `validate-engine-config`
- `cancel-execution`
- `check-cancellation`
- `observe-engine-event`
- `prepare-execution-graph`
- `execute-ordered-steps`
- `record-execution`
- `enforce-output-contract`
- `enforce-run-artifacts`
- `enforce-metrics-envelope`

## Contracts and operating rules

The crate root intentionally has only this `README.md`. All other crate docs live
under `docs/`, with a 10-document allowance:

- `docs/ARCHITECTURE.md`
- `docs/BOUNDARY.md`
- `docs/CHANGE_RULES.md`
- `docs/COMMANDS.md`
- `docs/DEPENDENCIES.md`
- `docs/DETERMINISM.md`
- `docs/EFFECTS.md`
- `docs/INDEX.md`
- `docs/PUBLIC_API.md`
- `docs/TESTS.md`

## Tests

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-engine --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-engine --no-default-features
```
