# `fastq.profile_read_lengths` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:26:51.299933+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_read_lengths/lunarc`
- Scenario: `read_length_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `seqkit_stats`
- report_only: `True`
- mutates_fastq: `False`
- may_change_read_count: `False`
- raw_backend_report_format: `seqkit_stats_length_histogram`
- histogram_bins: `100`

## Executive Summary

- Fastest median runtime: `seqkit_stats` at `1.646` seconds.
- Highest median max read length: `seqkit_stats` at `76.0`.
- Widest median distinct-length support: `seqkit_stats` at `7.0`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median read count | Median mean read length | Median max read length | Median distinct lengths |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `seqkit_stats` | 20 | 100.0% | 1.646 | 1241122.5 | 76.000 | 76.0 | 7.0 |

## Cohort Coverage

| Cohort | Samples |
| --- | ---: |
| `ancient_pe` | 5 |
| `ancient_se` | 5 |
| `modern_pe` | 5 |
| `modern_se` | 5 |

## Notes

- This stage is report-only and non-mutating: governed benchmarking confirms runtime and read-length distribution stability without changing the reads.
- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection.
