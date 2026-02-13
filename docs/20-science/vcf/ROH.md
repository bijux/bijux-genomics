# VCF ROH Stage

## Purpose
Define methodological intent for `vcf.roh` (runs of homozygosity) summaries.

## Scope
Applies to ROH segment detection and aggregate ROH burden metrics.

## Non-goals
- Claiming equivalence across heterogeneous ROH parameterizations.

## Contracts
- Stage contract: `domain/vcf/stages/roh.yaml`.
- Expected output: `roh_report`.
- Baseline planned tool: `plink2`.

## Validity Limits
- ROH sensitivity depends on marker density and genotype quality.
- Parameter drift invalidates direct cross-cohort comparison.
