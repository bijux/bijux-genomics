# VCF ROH Stage

## Purpose
Define the governed ROH inference boundary for runs-of-homozygosity summaries after cohort-level QC has stabilized the input matrix.

## Scope
This science surface covers:
- `vcf.qc` as the upstream missingness and marker-quality gate for interpretable ROH calling.
- `vcf.roh` as the supported interval-detection and aggregate burden stage.

## Non-goals
- Claiming equivalence across heterogeneous ROH parameterizations.
- Treating pre-QC and post-QC variant matrices as interchangeable ROH inputs.

## Contracts
- `vcf.roh` emits `roh_report` with schema `bijux.vcf.roh.v1`.
- The admitted and default backend is `plink2`, matching `domain/vcf/stages/roh.yaml` and `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- Required metrics include `sample_count`, `segment_count`, `total_length`, `segments`, `per_sample_summary`, and `status`.
- Every normalized segment row must keep `sample_id`, `contig`, `start`, `end`, `length`, and `variant_count`.
- ROH thresholds must stay traceable to the same QC-filtered matrix that passed `vcf.qc`.

## Validity Limits
- ROH sensitivity depends on marker density, genotype quality, and the missingness decisions applied in `vcf.qc`.
- Parameter drift invalidates direct cross-cohort comparison.
- ROH segment interpretation is only comparable when min-length, density, and pruning settings are held constant.
