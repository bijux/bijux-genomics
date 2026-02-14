# Run Output Contract

## What
Defines the expected output directory layout for runs.

## Why
Provides stable paths for analysis and benchmarking.

## Non-goals
- Custom directory layouts per user.

## Contracts
- Layout is derived from RunLayout.
- `explain.json` must include a decision trace for planner decisions that alter stage/tool behavior.
- For VCF calling, decision id `decision.coverage_regime` must be present with selected value in `{gl,diploid,pseudohaploid}`.
- Decision traces must include: decision id, selected value, evaluated evidence, and source config path.
- `explain.json` must include `coverage_regime.selected`, `coverage_regime.thresholds_used`, and `coverage_regime.observed_coverage_stats`.
- `report.json` must include `coverage_regime.selected`, `coverage_regime.thresholds_used`, and `coverage_regime.observed_coverage_stats` for VCF runs.
- VCF downstream outputs must include postprocess artifacts:
  - `vcf_postprocess/postprocess.vcf.gz`
  - `vcf_postprocess/postprocess.vcf.gz.tbi`
  - optional `vcf_postprocess/postprocess.bcf` when enabled
  - `vcf_postprocess/validate_outputs.json`
  - `vcf_postprocess/artifact_checksums.json`
- VCF runs with downstream stages must expose run-level checksums at `artifact_checksums.json`.
- `validate_outputs.json` must record readable VCF status, index presence, and contig consistency with `SpeciesContext`.

## Examples
- `run_artifacts/` lives under the run output directory.
- `explain.json` includes `decision_traces[].id = "decision.coverage_regime"` with coverage-derived regime.
- `vcf_postprocess/validate_outputs.json` reports `{readable_vcf, tabix_present, contigs_consistent_with_species_context}`.

## Failure modes
- Layout drift breaks replay and audits.
