# BACKENDS

This document defines backend invariants shared by docker and local execution.

## Invariants
- **cwd**: commands run from the stage output directory.
- **mounts**: input paths are mounted read-only; output paths are writable.
- **env**: environment variables are passed verbatim from the execution plan.
- **stdout/stderr**: captured separately and persisted for every step.
- **exit semantics**: non-zero exit code is treated as failure and recorded.

Backends must provide the same observable behavior for these invariants.
