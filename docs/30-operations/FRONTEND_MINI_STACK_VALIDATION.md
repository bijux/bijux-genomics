# Frontend Mini Stack Validation

## Purpose
Document mini/test-stack behavioral validation for FASTQ, BAM, and VCF frontend workflows.

## Scope
Frontend-only mini/test stacks executed by repository validation scripts.

## Non-goals
This document does not define heavy-corpus performance or production-scale acceptance criteria.

## Contracts
The mini validator must enforce all checks listed below against the runnable example surface in
[examples/index.yaml](../../examples/index.yaml).

## Runner
- Script: `cargo run -q -p bijux-dna-dev -- tooling run validate-frontend-mini-domain-stacks`
- Check wrapper: `cargo run -q -p bijux-dna-dev -- checks run check-frontend-mini-domain-validation`
- Output summary: `artifacts/domain/frontend-mini-validation/summary.json`

## What Is Validated
- FASTQ mini path (`fastq_edna_mini`) and VCF mini paths (`vcf_damage_aware_genotype_mini`, `vcf_downstream_vcf_full_mini`, `vcf_downstream_demography_mini`, `vcf_imputation_mini`) are executed through the example harness.
- Example outputs are compared against committed golden `plan.json`, `explain.json`, and
  `report.json` under the bundle rules defined in
  [EXAMPLE_RUNNER_CONTRACT.md](../50-reference/EXAMPLE_RUNNER_CONTRACT.md).
- Artifact bundle contract presence is checked (`plan/explain/report/run_report/metrics/logs`).
- `metrics.json` and `logs.txt` structural checks are enforced.
- VCF `coverage_regime` observability contract is enforced in
  [EXPLAINABILITY.md](EXPLAINABILITY.md) and [REPORT_CONTRACT.md](REPORT_CONTRACT.md).
- Coverage branching behavior is validated for:
  - `gl` (low depth)
  - `pseudohaploid` (mid depth)
  - `diploid` (high depth)
  via `cargo run -p bijux-dna-dev -- tooling run simulate-coverage-regime`.
- BAM authenticity consistency is validated from stage contract + fixtures (`authenticct`, `pmdtools`, `damageprofiler`).

## Current Gap (Detected by Validator)
- Bench-suite stage lists in `examples/*/bench-suite.toml` are not reflected in generated `artifacts/examples/*/plan.json` stage arrays for the mini harness.
- The validator treats this as a failing contract until the harness emits stage-by-stage plan coverage for those mini suites.
