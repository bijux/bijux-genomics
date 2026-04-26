# BOUNDARY

Owner: Data source
Scope: Reference database resolution and typed contract projection
Allowed inputs: read-only runtime config, declared reference paths, deterministic fixtures
Forbidden dependencies: planner orchestration, stage execution, runner backends, CLI adapters
Forbidden effects: network access, process spawning, writes outside caller-provided roots
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ref --no-default-features`

`bijux-dna-db-ref` is a pure resolution layer.

Allowed:
- Read-only config parsing from `configs/runtime/*`.
- Deterministic transformation to typed contracts.

Forbidden:
- Network access.
- Process spawning.
- Planner/runner side effects.

The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
