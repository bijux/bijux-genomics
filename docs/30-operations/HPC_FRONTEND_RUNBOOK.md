# HPC Frontend Runbook

## Purpose
Operational runbook for frontend-only (no Slurm) container builds, mini-runs, and validation.

## Scope
Defines required paths, permissions, frontend-only constraints, and the canonical validation command sequence.

## Non-goals
- Defining Slurm orchestration or compute-node execution policy.

## Contracts
- All frontend validation runs must execute from declared repo/output roots.
- Frontend validation must not use Slurm in this phase.
- Validation evidence must be written under documented artifact paths.

[../../containers/docs/FRONTEND_BUILD_AUTHORITY.md](../../containers/docs/FRONTEND_BUILD_AUTHORITY.md),
[TRACEABILITY_PROOF_FRONTEND.md](TRACEABILITY_PROOF_FRONTEND.md), and
[SLURM_PHASE_ENTRY_CRITERIA.md](SLURM_PHASE_ENTRY_CRITERIA.md) define the
adjacent control and proof surfaces for this frontend-only runbook.

## Paths
- SIF root: `${BIJUX_HPC_ROOT:-$HOME/bijux}/bijux-dna-containers/apptainer`
- Apptainer cache: `${BIJUX_HPC_ROOT:-$HOME/bijux}/bijux-dna-containers/cache`
- Output root: `${BIJUX_HPC_ROOT:-$HOME/bijux}/bijux-dna-results`
- Repo root: `${BIJUX_HPC_ROOT:-$HOME/bijux}/bijux-dna`

## Expected Permissions
- Repo and results dirs: user read/write/execute.
- Container dir: user read/write; group read optional.
- Cache dir: user read/write/execute; no world-write.
- Generated artifacts under `artifacts/` remain user-owned.

## Frontend-Only Rules
- Do not build on compute hosts.
- Do not use Slurm for mini validation in this phase.
- Enforce host policy via
  [configs/ci/tools/hpc_frontend_build_policy.toml](../../configs/ci/tools/hpc_frontend_build_policy.toml).

## Validation Commands
1. `cargo run -q -p bijux-dna-dev -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -p bijux-dna-dev -- containers run apptainer-build-all`
3. `cargo run -q -p bijux-dna-dev -- hpc run run-frontend-mini-e2e --confirm`
4. `cargo run -q -p bijux-dna-dev -- checks run check-frontend-mini-artifacts`
5. `cargo run -q -p bijux-dna-dev -- checks run check-frontend-observability-proof`
6. `cargo run -q -p bijux-dna-dev -- checks run check-frontend-telemetry-sanity`

## Artifacts
- Frontend smoke: `artifacts/containers/hpc/frontend-smoke/`
- Frontend mini E2E: `artifacts/hpc/frontend-mini-e2e/<run-id>/`
- Security/repro summaries:
  - [containers/docs/APPTAINER_FRONTEND_SECURITY_SUMMARY.md](../../containers/docs/APPTAINER_FRONTEND_SECURITY_SUMMARY.md)
  - [containers/docs/APPTAINER_FRONTEND_REPRODUCIBILITY_REPORT.md](../../containers/docs/APPTAINER_FRONTEND_REPRODUCIBILITY_REPORT.md)
