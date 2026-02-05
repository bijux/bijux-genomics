# bijux-core Scope

## What belongs in bijux-core
- Contract types that define the stable, serialized interfaces between crates.
- Execution plan types and validation logic.
- Domain-agnostic primitives: hashing, invariants, errors, input assessment.
- Small, pure helpers that are deterministic and side-effect free.

## What must NOT be added
- Runtime wiring (telemetry, observability, tracing, logging setup).
- IO, process execution, environment probing, or network access.
- Domain semantics (FASTQ/BAM/QC logic, stage registries, tool selection).
- Engine/runner orchestration, scheduling, or retry policy.
- Database access, web servers, or CLI concerns.

If a feature needs IO, runtime configuration, or domain semantics, it does not belong in core.
