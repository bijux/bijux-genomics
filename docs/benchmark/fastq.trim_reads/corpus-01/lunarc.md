# `fastq.trim_reads` on `corpus-01`

## Executive Summary

- `9` governed trim backends were benchmarked across `20` human samples (`180/180` zero-exit tool-sample observations).
- Fastest median runtime: `seqkit` at `1.216` seconds.
- Slowest median runtime: `atropos` at `24.170` seconds.
- Highest median base retention: `atropos` at `1.000`.
- Lowest median base retention: `fastp` at `0.918`.

## Run Contract

- Platform: `lunarc-apptainer`
- Stage: `fastq.trim_reads`
- Scenario: `trim_fairness`
- Tools: `adapterremoval, atropos, bbduk, cutadapt, fastp, prinseq, seqkit, trim_galore, trimmomatic`
- min_length: `30`
- quality_cutoff: `None`
- n_policy: `retain`
- adapter_policy: `none`
- polyx_policy: `none`
- contaminant_policy: `none`

## Tool Ranking

| Tool | Pass rate | Median runtime (s) | p90 runtime (s) | Median base retention | Median read retention | Mean Q delta | Slowdown vs fastest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `seqkit` | 100.0% | 1.216 | 7.261 | 1.000 | 1.000 | 0.000 | 1.00x |
| `prinseq` | 100.0% | 2.013 | 14.939 | 1.000 | 1.000 | 0.000 | 1.66x |
| `adapterremoval` | 100.0% | 3.198 | 23.735 | 0.980 | 0.998 | 0.000 | 2.63x |
| `bbduk` | 100.0% | 3.199 | 12.107 | 1.000 | 1.000 | 0.000 | 2.63x |
| `fastp` | 100.0% | 5.523 | 18.562 | 0.918 | 0.948 | 0.000 | 4.54x |
| `cutadapt` | 100.0% | 5.794 | 50.680 | 1.000 | 1.000 | 0.000 | 4.76x |
| `trimmomatic` | 100.0% | 16.225 | 107.191 | 1.000 | 1.000 | 0.000 | 13.34x |
| `trim_galore` | 100.0% | 20.104 | 166.891 | 1.000 | 1.000 | 0.000 | 16.53x |
| `atropos` | 100.0% | 24.170 | 243.311 | 1.000 | 1.000 | 0.000 | 19.87x |

## Cohort Behavior

- For `fastp`, `modern_pe` samples ran at `16.128` seconds median versus `5.770` for `ancient_se`.
- The fastest backend `seqkit` and highest-retention backend `atropos` are different, which matters when choosing a default objective.

## Highest-Cost Samples

| Sample | Accession | Cohort | Total runtime (s) | Slowest tool | Slowest runtime (s) | Lowest-retention tool | Lowest base retention |
| --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0006` | `DRR001073` | `modern_se` | 680.220 | `atropos` | 243.311 | `fastp` | 0.683 |
| `sample_0018` | `ERR769591` | `ancient_se` | 675.901 | `atropos` | 319.493 | `adapterremoval` | 0.994 |
| `sample_0003` | `DRR000550` | `modern_pe` | 656.824 | `atropos` | 215.798 | `fastp` | 0.919 |
| `sample_0001` | `DRR000093` | `modern_pe` | 603.828 | `atropos` | 204.865 | `fastp` | 0.952 |
| `sample_0002` | `DRR000095` | `modern_pe` | 591.623 | `atropos` | 200.568 | `fastp` | 0.943 |

## Interpretation

- This corpus benchmark is intentionally bank-free for trim adapters/polyX/contaminants so the governed fairness cohort stays comparable across all included backends.
- The strongest choice for production depends on whether we prefer latency, base retention, or quality uplift; this dossier exposes those tradeoffs instead of collapsing them into one score.

## Reproducibility

- `summary.json`, `sample_results.csv`, and the CSV analysis tables in this directory are generated artifacts from the same corpus run.
- `sample_results.csv` preserves one row per sample/tool execution for independent reanalysis.
