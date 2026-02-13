# VCF Demography Stage

## Purpose
Define methodological intent for `vcf.demography` inference from IBD summaries.

## Scope
Applies to recent effective population size style summaries derived from IBD inputs.

## Non-goals
- Long-term demographic model fitting beyond current stage contracts.

## Contracts
- Stage contract: `domain/vcf/stages/demography.yaml`.
- Expected output: `demography_report`.
- Baseline planned tool: `ibdne`.
- Output contract requires `metrics.json` with schema `bijux.vcf.demography.v1`.
- Required metrics include `ne_recent`, `ne_time_series`, confidence intervals, and explicit assumption flags.

## Validity Limits
- Requires stable upstream IBD calling assumptions.
- Ne estimates are model-dependent and should be interpreted with confidence ranges.
- Generation time and recombination assumptions must be fixed and reported per run.
