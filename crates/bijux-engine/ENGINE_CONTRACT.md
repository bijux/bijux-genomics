# ENGINE_CONTRACT

Engine consumes an `ExecutionGraph` and a `Runner`.
It must:
- Validate graph structure.
- Enforce contracts after each step.
- Emit deterministic ordering when configured.
