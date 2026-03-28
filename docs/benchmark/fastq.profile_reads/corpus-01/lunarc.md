# `fastq.profile_reads` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:26:36.651353+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_reads/lunarc`
- Scenario: `profile_reads_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `seqkit_stats`
- report_only: `True`
- mutates_fastq: `False`
- may_change_read_count: `False`
- raw_backend_report_format: `seqkit_stats_tsv`
- length_histogram_source: `seqkit_fx2tab`

## Executive Summary

- Fastest median runtime: `seqkit_stats` at `1.614` seconds.
- Highest median mean Q: `seqkit_stats` at `0.000`.
- Widest median histogram support: `seqkit_stats` at `7.0` bins.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median reads | Median bases | Median mean Q | Median GC % | Median read length | Median histogram bins |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `seqkit_stats` | 20 | 100.0% | 1.614 | 1241122.5 | 78085583.5 | 0.000 | 46.540 | 76.000 | 7.0 |

## Cohort Coverage

| Cohort | Samples |
| --- | ---: |
| `ancient_pe` | 5 |
| `ancient_se` | 5 |
| `modern_pe` | 5 |
| `modern_se` | 5 |

## Notes

- This stage is report-only and non-mutating: governed benchmarking confirms runtime and profile-report stability without changing the reads.
- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection.
