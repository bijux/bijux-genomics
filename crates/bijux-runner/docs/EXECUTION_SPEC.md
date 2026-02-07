# EXECUTION_SPEC

## Execution semantics
- cwd: absolute working directory for process execution
- env: allowlisted environment variables
- mounts: explicit host paths only
- stdout/stderr: captured and stored per step
- exit semantics: zero == success, non-zero == ToolError

## Hashing
Invocation hash includes command, args, env, cwd, mounts, and image digest.
