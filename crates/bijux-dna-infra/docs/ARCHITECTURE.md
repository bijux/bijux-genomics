# Architecture

## Modules
- formats/
- io/
- locking.rs
- logging/
- paths/
- retry/
- run_directories/
- temp.rs

## Data flow
- `paths/` owns deterministic path construction only.
- `run_directories/` owns run-layout contracts plus lock/publish operations.
- `io/` owns filesystem effects and the infra-local IO error taxonomy.
- `retry/` owns retry policy, clock abstraction, backoff math, and retry execution.
- `logging/` owns tracing bootstrap and subscriber wiring.
- No module may introduce domain semantics or become a generic catch-all.
