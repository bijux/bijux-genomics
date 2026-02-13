# Apptainer Frontend Reproducibility

Purpose: enforce deterministic Apptainer SIF rebuild behavior on HPC frontend nodes.

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
  - `scripts/containers/check-version-hash-pin.sh` must pass before rebuild sampling.
- Compute-node refusal:
  - hostname policy from `configs/ci/tools/hpc_frontend_build_policy.toml`.

## Acceptance Standard
- Config: `configs/ci/tools/apptainer_reproducibility_policy.toml`
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
  - `./bin/isolate ./scripts/containers/run-apptainer-frontend-reproducibility.sh`
- Gate check:
  - `./scripts/containers/check-apptainer-frontend-reproducibility.sh`

## Outputs
- Machine summary:
  - `artifacts/containers/hpc/frontend-reproducibility/<run_id>/summary.json`
- Human report:
  - `containers/docs/APPTAINER_FRONTEND_REPRODUCIBILITY_REPORT.md`

## Nondeterminism Cause Labels
- `timestamp_or_timezone`
- `tar_or_archive_order`
- `compiler_or_toolchain`
- `unknown_or_external_dependency_drift`
