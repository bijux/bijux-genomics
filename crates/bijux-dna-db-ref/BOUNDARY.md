# BOUNDARY

`bijux-dna-db-ref` is a pure resolution layer.

Allowed:
- Read-only config parsing from `configs/runtime/*`.
- Deterministic transformation to typed contracts.

Forbidden:
- Network access.
- Process spawning.
- Planner/runner side effects.
