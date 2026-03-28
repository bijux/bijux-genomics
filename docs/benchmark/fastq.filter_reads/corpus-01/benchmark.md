# `fastq.filter_reads` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.filter_reads` stage across the full corpus-01 human DNA cohort on the Lunarc Apptainer platform.

## Executive Summary

- `4` governed filter backends were benchmarked across `20` samples (`80/80` zero-exit tool-sample observations).
- Fastest median runtime: `seqkit` at `1.239` seconds.
- Highest median base retention: `bbduk` at `100.0%`.
- Highest mean reads dropped: `fastp` at `284465.7`.

## Run Contract

- Platform: `lunarc-apptainer`
- Stage: `fastq.filter_reads`
- Scenario: `filter_fairness`
- Tools: `bbduk, fastp, prinseq, seqkit`
- max_n: `0`
- max_n_count: `3`
- low_complexity_threshold: `20.0`
- entropy_threshold: `18.0`
- kmer_ref: `None`
- polyx_policy: `trim`

## Tool Ranking

| Tool | Median runtime (s) | p90 runtime (s) | Median base retention | Median read retention | Mean reads dropped | Mean low-complexity removals | Mean N removals | Mean Q delta | Slowdown vs fastest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `bbduk` | 3.445 | 12.426 | 100.0% | 100.0% | 0.0 | 0.0 | 0.0 | 0.000 | x2.78 |
| `fastp` | 4.150 | 9.980 | 92.0% | 95.2% | 284465.7 | 0.0 | 1265.3 | 0.000 | x3.35 |
| `prinseq` | 1.962 | 8.334 | 100.0% | 100.0% | 0.0 | 0.0 | 0.0 | 0.000 | x1.58 |
| `seqkit` | 1.239 | 4.273 | 100.0% | 100.0% | 0.0 | 0.0 | 0.0 | 0.000 | x1.00 |

## Cohort Behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median base retention | Median read retention | Mean reads dropped | Samples |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `bbduk` | `ancient_pe` | 3.495 | 1.821 | 100.0% | 100.0% | 0.0 | 5 |
| `bbduk` | `ancient_se` | 5.959 | 3.024 | 100.0% | 100.0% | 0.0 | 5 |
| `bbduk` | `modern_pe` | 6.589 | 9.516 | 100.0% | 100.0% | 0.0 | 5 |
| `bbduk` | `modern_se` | 9.055 | 3.867 | 100.0% | 100.0% | 0.0 | 5 |
| `bbduk` | `under_1000mb` | 23.793 | 23.793 | 100.0% | 100.0% | 0.0 | 1 |
| `bbduk` | `under_100mb` | 2.255 | 1.898 | 100.0% | 100.0% | 0.0 | 12 |
| `bbduk` | `under_500mb` | 10.662 | 10.265 | 100.0% | 100.0% | 0.0 | 7 |
| `fastp` | `ancient_pe` | 2.736 | 1.786 | 75.4% | 97.6% | 76660.8 | 5 |
| `fastp` | `ancient_se` | 8.025 | 6.537 | 100.0% | 100.0% | 55.6 | 5 |
| `fastp` | `modern_pe` | 5.247 | 7.479 | 91.8% | 91.8% | 504247.2 | 5 |
| `fastp` | `modern_se` | 7.881 | 3.952 | 90.7% | 91.7% | 556899.2 | 5 |
| `fastp` | `under_1000mb` | 18.659 | 18.659 | 68.2% | 68.2% | 2391172.0 | 1 |
| `fastp` | `under_100mb` | 3.156 | 3.281 | 92.5% | 97.9% | 21973.8 | 12 |
| `fastp` | `under_500mb` | 8.988 | 7.835 | 91.8% | 94.7% | 433493.7 | 7 |
| `prinseq` | `ancient_pe` | 2.346 | 1.012 | 100.0% | 100.0% | 0.0 | 5 |
| `prinseq` | `ancient_se` | 4.113 | 1.762 | 100.0% | 100.0% | 0.0 | 5 |
| `prinseq` | `modern_pe` | 4.869 | 7.314 | 100.0% | 100.0% | 0.0 | 5 |
| `prinseq` | `modern_se` | 5.618 | 2.162 | 100.0% | 100.0% | 0.0 | 5 |
| `prinseq` | `under_1000mb` | 15.949 | 15.949 | 100.0% | 100.0% | 0.0 | 1 |
| `prinseq` | `under_100mb` | 1.193 | 1.083 | 100.0% | 100.0% | 0.0 | 12 |
| `prinseq` | `under_500mb` | 7.781 | 7.314 | 100.0% | 100.0% | 0.0 | 7 |
| `seqkit` | `ancient_pe` | 1.151 | 0.607 | 100.0% | 100.0% | 0.0 | 5 |
| `seqkit` | `ancient_se` | 2.135 | 1.084 | 100.0% | 100.0% | 0.0 | 5 |
| `seqkit` | `modern_pe` | 2.408 | 3.630 | 100.0% | 100.0% | 0.0 | 5 |
| `seqkit` | `modern_se` | 3.299 | 1.395 | 100.0% | 100.0% | 0.0 | 5 |
| `seqkit` | `under_1000mb` | 8.911 | 8.911 | 100.0% | 100.0% | 0.0 | 1 |
| `seqkit` | `under_100mb` | 0.792 | 0.652 | 100.0% | 100.0% | 0.0 | 12 |
| `seqkit` | `under_500mb` | 3.792 | 3.721 | 100.0% | 100.0% | 0.0 | 7 |

## Highest-Cost Samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest filter tool | Reads dropped |
| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 67.312 | `bbduk` | 23.793 | `fastp` | 2391172.0 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 59.141 | `fastp` | 19.094 | `fastp` | 226.0 |
| `sample_0008` | `DRR001083` | `modern` | `se` | `under_500mb` | 33.932 | `bbduk` | 12.426 | `fastp` | 188970.0 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 31.967 | `bbduk` | 10.633 | `fastp` | 1100272.0 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 29.426 | `bbduk` | 10.265 | `fastp` | 655136.0 |

## Interpretation

- Because `corpus-01` is a human DNA cohort rather than a synthetic junk-read challenge set, aggressive filtering here should be weighed carefully against retention loss.
- The published CSV artifacts preserve per-tool removal counters so later audits can distinguish N-filtering, low-complexity filtering, and backend-specific retention behavior.
