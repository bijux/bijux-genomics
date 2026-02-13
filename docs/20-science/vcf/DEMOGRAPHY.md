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

## Validity Limits
- Requires stable upstream IBD calling assumptions.
- Ne estimates are model-dependent and should be interpreted with confidence ranges.
