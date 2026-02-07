# CONTRACT

## Boundary: Stage Plan vs Run Manifest
This crate defines **planning contracts** only:

- Stage plans describe *what* should run and *what artifacts are expected*.
- Run manifests (runtime) describe *what actually ran* and *what artifacts were produced*.

A stage plan never includes execution outcomes, timestamps, exit codes, or runtime paths.
A run manifest never defines planning intent or tool selection logic.

## Non-goals
- Tool execution details
- Runtime records
- Artifact validation
