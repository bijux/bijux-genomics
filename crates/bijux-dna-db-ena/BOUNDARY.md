# BOUNDARY

Owner: Data source
Scope: ENA metadata selection, query models, and download client boundary
Allowed inputs: ENA request parameters, caller-provided output roots, deterministic fixtures
Forbidden dependencies: planner orchestration, stage execution, runner backends, CLI adapters
Forbidden effects: writes outside caller-provided roots, hidden pipeline execution, undeclared network use
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ena --no-default-features`

## What this crate owns
- Typed ENA metadata/query models.
- ENA selection and download client logic.

## What this crate must not do
- Must not hardcode host-specific paths.
- Must not write artifacts outside caller-provided output roots.
- Must not own pipeline planning or stage execution logic.
