# HPC Frontend Runbook

## Purpose
Operational runbook for frontend-only (no Slurm) container builds, mini-runs, and validation.

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
- Enforce host policy via `configs/ci/tools/hpc_frontend_build_policy.toml`.

## Validation Commands
1. `./scripts/hpc/validate-frontend-constraints.sh --confirm`
2. `./scripts/containers/apptainer-build-all.sh`
3. `./scripts/hpc/run-frontend-mini-e2e.sh --confirm`
4. `./scripts/checks/check-frontend-mini-artifacts.sh`
5. `./scripts/checks/check-frontend-observability-proof.sh`
6. `./scripts/checks/check-frontend-telemetry-sanity.sh`

## Artifacts
- Frontend smoke: `artifacts/containers/hpc/frontend-smoke/`
- Frontend mini E2E: `artifacts/hpc/frontend-mini-e2e/<run-id>/`
- Security/repro summaries:
  - `containers/docs/APPTAINER_FRONTEND_SECURITY_SUMMARY.md`
  - `containers/docs/APPTAINER_FRONTEND_REPRODUCIBILITY_REPORT.md`
