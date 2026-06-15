# VCF Demography Stage

## Purpose
Define the governed recent-demography inference boundary that consumes IBD summaries instead of pretending effective-size outputs are derivable directly from raw cohort VCF input.

## Scope
This science surface covers:
- `vcf.ibd` as the required upstream segment summary contract.
- `vcf.demography` as the planned recent-Ne style inference stage built on those IBD summaries.

## Non-goals
- Long-term demographic model fitting beyond current stage contracts.
- Treating `vcf.demography` as valid when upstream `vcf.ibd` assumptions drift unreported.

## Contracts
- `vcf.demography` emits `demography_report` with schema `bijux.vcf.demography.v1`.
- The admitted and default backend is `ibdne`, matching `domain/vcf/stages/demography.yaml` and `domain/vcf/docs/DEFAULT_SETTINGS.md`, while runtime packaging remains planned.
- Required metrics include `method`, `inference_status`, `status`, `insufficient_reason`, `time_bins`, `ne_estimates`, and `insufficient_data_probe`.
- The stage must preserve enough provenance to identify which `vcf.ibd` backend and segment thresholds produced the demography input.

## Validity Limits
- Validity requires stable upstream `vcf.ibd` calling assumptions and a fixed segment-length regime.
- Ne estimates are model-dependent and should be interpreted with confidence ranges, not as absolute demographic truth.
- Generation time and recombination assumptions must be fixed and reported per run.
