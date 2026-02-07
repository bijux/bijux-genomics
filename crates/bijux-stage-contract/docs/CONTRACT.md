# CONTRACT

This crate owns:
- StagePlanV1
- ExecutionPlanV1
- StagePlugin wiring types
- Plan-scoped Run context

It must remain minimal and dependency-light. Any overlap with core run
contracts should be resolved in favor of bijux-core; if duplication appears,
rename plan-local types (e.g., plan_run) or relocate them.

## Versioning Rules

Breaking changes (field removal/rename/type changes or semantic shifts) require
bumping the major version in schema identifiers and any documented contract
version. Non-breaking additions may bump minor versions.
