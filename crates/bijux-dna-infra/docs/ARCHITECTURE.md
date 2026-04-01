# Architecture

## Modules
- formats/
- hashing/
- io/
- locking.rs
- logging/
- paths/
- retry/
- run_directories/
- temp/

## Data flow
- `lib.rs` stays explicit about the public surface while delegating export ownership to `stable_surface.rs`.
- `formats/`, `io/`, `paths/`, `retry/`, and `run_directories/` keep their stable exports in dedicated `stable_surface.rs` files.
- `hashing/` owns file hashing entrypoints and delegates file digest IO to a companion module.
- `paths/` owns deterministic path construction only.
- `run_directories/` owns run-layout contracts plus lock/publish operations.
- `io/` owns filesystem effects and the infra-local IO error taxonomy.
- `retry/` owns retry policy, clock abstraction, backoff math, and retry execution.
- `logging/` owns tracing bootstrap and subscriber wiring.
- `temp/` owns temporary-directory entrypoints.
- No module may introduce domain semantics or become a generic catch-all.
