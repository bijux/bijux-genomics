# Frontend Mini Stack Validation

Purpose: document mini/test-stack behavioral validation for FASTQ, BAM, and VCF frontend workflows.

## Runner
- Script: `scripts/tooling/validate-frontend-mini-domain-stacks.sh`
- Check wrapper: `scripts/checks/check-frontend-mini-domain-validation.sh`
- Output summary: `artifacts/domain/frontend-mini-validation/summary.json`

## What Is Validated
- FASTQ mini path (`fastq_edna_mini`) and VCF mini paths (`vcf_damage_aware_genotype_mini`, `vcf_downstream_vcf_full_mini`, `vcf_downstream_demography_mini`, `vcf_imputation_mini`) are executed through the example harness.
- Example outputs are compared against committed golden `plan.json`, `explain.json`, and `report.json`.
- Artifact bundle contract presence is checked (`plan/explain/report/run_report/metrics/logs`).
- `metrics.json` and `logs.txt` structural checks are enforced.
- VCF `coverage_regime` observability contract is enforced in explain/report.
- Coverage branching behavior is validated for:
  - `gl` (low depth)
  - `pseudohaploid` (mid depth)
  - `diploid` (high depth)
  via `scripts/tooling/simulate-coverage-regime.sh`.
- BAM authenticity consistency is validated from stage contract + fixtures (`authenticct`, `pmdtools`, `damageprofiler`).

## Current Gap (Detected by Validator)
- Bench-suite stage lists in `examples/*/bench-suite.toml` are not reflected in generated `artifacts/examples/*/plan.json` stage arrays for the mini harness.
- The validator treats this as a failing contract until the harness emits stage-by-stage plan coverage for those mini suites.
