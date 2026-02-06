# ARCHITECTURE

Public surface:
- `Engine`, `EngineConfig`, `EngineError`.

Core module:
- `executor.rs` orchestrates execution.

Runtime services live in `services/runtime_services.rs` and are internal.
