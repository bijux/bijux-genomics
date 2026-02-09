# METRICS

Retention must include:
- numerator
- denominator
- units
- conditions

Truthful retention requires recording context.

## Typed QC summarize outputs
Typed QC/summarize schemas are defined in `src/metrics/types.rs`:
- `FastqScanMetricsV1`
- `SeqfuMetricsV1`

Canonical required units/fields:
- read count (`reads`)
- base count in bp (`bases_bp`)
- mean read length in bp (`mean_read_length_bp`)
- Q-score summary in Phred (`mean_phred`, `median_phred`, `p10_phred`, `p90_phred`)
- duplication estimate percentage when available (`duplication_estimate_pct`)

## Typed screening/classification outputs
Screening outputs are modeled with:
- `KrakenUniqClassificationMetricsV1`
- `BrackenClassificationMetricsV1`
- shared `TaxonomyRecordV1`
- required DB provenance `ClassificationDbProvenanceV1` (`db_name`, `db_version`, `db_hash`)

KrakenUniq-specific fields include unique-kmer counts and optional confidence.
Bracken-specific fields include estimated abundances.
