# BOUNDARY

Environment resolution is configuration-only.
The runner executes tools; environment never does.

## Allowed
- Parsing and validating specs.
- Resolving image references.

## Forbidden
- Executing tools.
- Spawning processes.
- Using `bijux-runner` or any execution backend.
