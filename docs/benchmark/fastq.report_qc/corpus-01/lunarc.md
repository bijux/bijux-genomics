# `fastq.report_qc` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:27:47.680379+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/System/Volumes/Data/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.report_qc/lunarc`
- Scenario: `qc_aggregation_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `multiqc`
- aggregation_engine: `multiqc`
- aggregation_scope: `governed_qc_artifacts`
- Governed contributor stages: `fastq.detect_adapters, fastq.profile_read_lengths, fastq.profile_reads, fastq.validate_reads`

## Executive Summary

- Fastest median runtime: `multiqc` at `2.276` seconds.
- Highest median MultiQC module count: `multiqc` at `11.0`.
- Highest median governed QC input count: `multiqc` at `6.0`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median modules | Median sample count | Median governed inputs | Median contamination rate | Median mean Q |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `multiqc` | 20 | 100.0% | 2.276 | 11.0 | 1.5 | 6.0 | 0.0000 | 0.000 |

## Cohort Coverage

| Cohort | Samples |
| --- | ---: |
| `ancient_pe` | 5 |
| `ancient_se` | 5 |
| `modern_pe` | 5 |
| `modern_se` | 5 |

## Notes

- This stage is report-only and non-mutating: governed benchmarking confirms MultiQC aggregation behavior without changing the reads.
- `sample_results.csv` beside this report keeps the per-sample MultiQC counts, contamination carry-through, and governed manifest lineage for deeper inspection.
