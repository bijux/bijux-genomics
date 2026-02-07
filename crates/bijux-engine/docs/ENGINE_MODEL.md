# ENGINE_MODEL

## Responsibilities
- Orchestrate execution of an `ExecutionGraph`.
- Enforce contracts after each step.
- Record the truth artifacts for each step.
- Emit structured events for observers.

## Non-responsibilities
- No process spawning.
- No Docker/local execution logic.
- No tool selection or planning.

## Boundaries
- Depends on `bijux-core` contracts and `bijux-runtime::Runner` trait.
- Invokes a runner to execute command specs.
