# Apptainer Frontend Reproducibility

Purpose: enforce deterministic Apptainer SIF rebuild behavior on HPC frontend nodes.

[FRONTEND_BUILD_AUTHORITY.md](FRONTEND_BUILD_AUTHORITY.md),
[../../docs/30-operations/HPC_FRONTEND_RUNBOOK.md](../../docs/30-operations/HPC_FRONTEND_RUNBOOK.md),
and [APPTAINER_FRONTEND_REPRODUCIBILITY_REPORT.md](APPTAINTER_FRONTEND_REPRODUCIBILITY_REPORT.md)
define the adjacent control and reporting surfaces for this reproducibility
gate.

## Scope
- Build authority is frontend/login hosts only.
- Sample 10 random tool definitions per run.
- For each tool, compare four builds:
  - baseline build 1
  - baseline build 2 (same cache)
  - post `apptainer cache clean -f`
  - post full cache purge (`rm -rf $APPTAINER_CACHEDIR $APPTAINER_TMPDIR`)

## Determinism Controls
- Environment normalization:
  - `TZ=UTC`
  - `LC_ALL=C`
  - `LANG=C`
  - `SOURCE_DATE_EPOCH` from repository HEAD commit time (or `0` fallback)
- Frontend pinned-version requirement:
  - `cargo run -p bijux-dna-dev -- containers run check-version-hash-pin` must pass before rebuild sampling.
- Compute-node refusal:
  - hostname policy from
    [configs/ci/tools/hpc_frontend_build_policy.toml](../../configs/ci/tools/hpc_frontend_build_policy.toml).

## Acceptance Standard
- Config:
  [configs/ci/tools/apptainer_reproducibility_policy.toml](../../configs/ci/tools/apptainer_reproducibility_policy.toml)
- Current standard:
  - `tool_sample_count = 10`
  - `confidence_min = 1.0`
  - `require_all_tools_deterministic = true`
- Confidence formula:
  - `passed_checks / total_checks`
  - `total_checks = sampled_tools * 3`
  - per-tool checks: `same_cache_twice`, `clean_cache_match`, `purge_cache_match`

## Commands
- Run workflow:
  - `cargo run -p bijux-dna-dev -- containers run run-apptainer-frontend-reproducibility`
- Gate check:
  - `cargo run -p bijux-dna-dev -- containers run check-apptainer-frontend-reproducibility`

## Outputs
- Machine summary:
  - `artifacts/containers/hpc/frontend-reproducibility/<run_id>/summary.json`
- Human report:
  - [containers/docs/APPTAINTER_FRONTEND_REPRODUCIBILITY_REPORT.md](APPTAINTER_FRONTEND_REPRODUCIBILITY_REPORT.md)

## Nondeterminism Cause Labels
- `timestamp_or_timezone`
- `tar_or_archive_order`
- `compiler_or_toolchain`
- `unknown_or_external_dependency_drift`
