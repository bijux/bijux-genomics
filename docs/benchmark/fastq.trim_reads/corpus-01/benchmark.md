# `fastq.trim_reads` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:46:45.334156+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_reads/lunarc`
- Scenario: `trim_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `adapterremoval, atropos, bbduk, cutadapt, fastp, prinseq, seqkit, trim_galore, trimmomatic`
- min_length: `30`
- quality_cutoff: `governed tool default`
- n_policy: `retain`
- adapter_policy: `none`
- polyx_policy: `none`
- contaminant_policy: `none`

## Executive Summary

- Fastest median runtime: `seqkit` at `1.216` seconds.
- Highest median base retention: `atropos` at `1.000`.
- Highest median read retention: `atropos` at `1.000`.
- Highest mean Q delta: `adapterremoval` at `0.000`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median base retention | Median read retention | Mean Q delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `adapterremoval` | 20 | 100.0% | 3.198 | 0.980 | 0.998 | 0.000 |
| `atropos` | 20 | 100.0% | 24.170 | 1.000 | 1.000 | 0.000 |
| `bbduk` | 20 | 100.0% | 3.199 | 1.000 | 1.000 | 0.000 |
| `cutadapt` | 20 | 100.0% | 5.794 | 1.000 | 1.000 | 0.000 |
| `fastp` | 20 | 100.0% | 5.523 | 0.918 | 0.948 | 0.000 |
| `prinseq` | 20 | 100.0% | 2.013 | 1.000 | 1.000 | 0.000 |
| `seqkit` | 20 | 100.0% | 1.216 | 1.000 | 1.000 | 0.000 |
| `trim_galore` | 20 | 100.0% | 20.104 | 1.000 | 1.000 | 0.000 |
| `trimmomatic` | 20 | 100.0% | 16.225 | 1.000 | 1.000 | 0.000 |

## Cohort Coverage

| Cohort | Samples |
| --- | ---: |
| `ancient_pe` | 5 |
| `ancient_se` | 5 |
| `modern_pe` | 5 |
| `modern_se` | 5 |

## Notes

- This benchmark intentionally pins bank-free trim policies so the full governed trim fairness cohort can execute under one comparable contract.
- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection.
