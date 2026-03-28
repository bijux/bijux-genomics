# `fastq.profile_overrepresented_sequences` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:27:05.660482+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_overrepresented_sequences/lunarc`
- Scenario: `overrepresented_sequence_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `fastq_scan, fastqc, seqkit`
- report_only: `True`
- mutates_fastq: `False`
- may_change_read_count: `False`
- top_k: `50`

## Executive Summary

- Fastest median runtime: `fastq_scan` at `1.259` seconds.
- Highest median profiled sequence count: `fastq_scan` at `50.0`.
- Highest median dominant-sequence fraction: `fastq_scan` at `0.002`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median sequence count | Median flagged sequences | Median top fraction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fastq_scan` | 20 | 100.0% | 1.259 | 50.0 | 0.0 | 0.002 |
| `fastqc` | 20 | 100.0% | 7.089 | 50.0 | 0.0 | 0.002 |
| `seqkit` | 20 | 100.0% | 1.961 | 50.0 | 0.0 | 0.002 |

## Cohort Coverage

| Cohort | Samples |
| --- | ---: |
| `ancient_pe` | 5 |
| `ancient_se` | 5 |
| `modern_pe` | 5 |
| `modern_se` | 5 |

## Notes

- This stage is report-only and non-mutating: governed benchmarking compares overrepresented-sequence ranking behavior without changing the reads.
- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection.
