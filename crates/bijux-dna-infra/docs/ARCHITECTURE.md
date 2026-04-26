# Architecture

`bijux-dna-infra` owns low-level infrastructure helpers used by higher DNA
crates. It provides deterministic formats, hashing, IO, paths, retry,
run-directory, logging, locking, and temp-directory utilities without owning
domain semantics.

## Root Layout

- `Cargo.toml` declares the low-level dependency graph and optional format
  features.
- `README.md` is the only root documentation file.
- `docs/` contains the 10 authoritative crate docs.
- `src/` contains the library implementation.
- `tests/` contains boundary, contract, determinism, guardrail, schema, and
  snapshot coverage.

## Source Layout

- `src/lib.rs` stays explicit about the public surface while delegating curated
  export ownership to `stable_surface.rs`.
- `src/formats/` owns JSON and YAML encoding boundaries.
- `src/hashing/` owns file hashing entrypoints and delegates digest IO to a
  companion module.
- `src/io/` owns filesystem effects and the infra-local IO error taxonomy.
- `src/locking.rs` owns cross-process lock helpers.
- `src/logging/` owns tracing bootstrap and subscriber wiring.
- `src/paths/` owns deterministic path construction only.
- `src/retry/` owns retry policy, clock abstraction, backoff math, and retry
  execution.
- `src/run_directories/` owns run-layout contracts plus lock/publish
  operations.
- `src/temp.rs` owns temporary-directory entrypoints.

## Test Layout

- `tests/boundaries/` protects architecture, docs layout, dependency ownership,
  policy guardrails, and canonical owner rules.
- `tests/contracts/` exercises IO and run-layout contracts.
- `tests/determinism/` checks hashing and retry determinism.
- `tests/schemas/` locks the public surface.
- `tests/snapshots/` holds governed snapshots only.

## Dependency Direction

Infra may depend on general-purpose crates needed for filesystem, hashing,
format, tracing, and retry behavior. It must not depend on domain, planner, API,
engine, runner, runtime, CLI, or development automation crates.

## Change Rules

- Keep stable exports in dedicated `stable_surface.rs` files.
- Keep filesystem effects under `io/`, `run_directories/`, `locking.rs`, and
  `temp.rs`.
- Keep deterministic path construction under `paths/`; do not add domain
  semantics there.
- Add new files only for durable low-level infrastructure concerns.
- Update `tests/boundaries/architecture.rs` and this document together when the
  tree changes intentionally.
