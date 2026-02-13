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
- Output contract requires `metrics.json` with schema `bijux.vcf.roh.v1`.
- Required metrics include ROH count, total Mb, length-bin histogram, mean length, and max length.

## Validity Limits
- ROH sensitivity depends on marker density and genotype quality.
- Parameter drift invalidates direct cross-cohort comparison.
- ROH bin interpretation is only comparable when min-length/density settings are held constant.
