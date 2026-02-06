# RUNTIME_CONTRACT

Runtime contracts define the stable interface between orchestration and
execution transports.

Owned contracts:
- Run layout (paths + locations).
- Recording outputs (run manifest, tool invocation, execution records).
- Runner trait (execution boundary).
- Telemetry + observability wiring (optional, transport-agnostic).

Non-goals:
- Tool selection, pipeline planning, or domain semantics.
- Storage backends beyond generic path-based I/O.
