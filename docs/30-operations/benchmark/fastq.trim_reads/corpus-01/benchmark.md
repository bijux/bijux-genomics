# `fastq.trim_reads` on `corpus-01`

## Executive Summary

- `13` governed trim backends were benchmarked across `20` human samples (`233/260` zero-exit tool-sample observations).
- Fastest median runtime: `seqkit` at `1.249` seconds.
- Slowest median runtime: `leehom` at `53.858` seconds.
- Highest median base retention: `atropos` at `1.000`.
- Lowest median base retention: `trimmomatic` at `0.000`.

## Run Contract

- Platform: `lunarc-apptainer`
- Stage: `fastq.trim_reads`
- Scenario: `trim_fairness`
- Tools: `adapterremoval, alientrimmer, atropos, bbduk, cutadapt, fastp, fastx_clipper, leehom, prinseq, seqkit, skewer, trim_galore, trimmomatic`
- min_length: `governed tool default`
- quality_cutoff: `governed tool default`
- n_policy: `retain`
- adapter_policy: `none`
- polyx_policy: `none`
- contaminant_policy: `none`

## Tool Ranking

| Tool | Pass rate | Median runtime (s) | p90 runtime (s) | Median base retention | Median read retention | Mean Q delta | Slowdown vs fastest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `seqkit` | 100.0% | 1.249 | 7.367 | 1.000 | 1.000 | 0.000 | 1.00x |
| `prinseq` | 100.0% | 2.042 | 15.567 | 1.000 | 1.000 | 0.000 | 1.64x |
| `adapterremoval` | 100.0% | 3.332 | 25.217 | 0.989 | 1.000 | 0.000 | 2.67x |
| `bbduk` | 100.0% | 3.422 | 12.224 | 1.000 | 1.000 | 0.000 | 2.74x |
| `fastp` | 100.0% | 5.547 | 17.580 | 0.920 | 0.956 | 0.000 | 4.44x |
| `cutadapt` | 100.0% | 6.072 | 48.809 | 1.000 | 1.000 | 0.000 | 4.86x |
| `alientrimmer` | 65.0% | 9.257 | 18.079 | 0.135 | 0.083 | 0.000 | 7.41x |
| `skewer` | 100.0% | 18.486 | 79.595 | 1.000 | 1.000 | 0.000 | 14.80x |
| `trim_galore` | 100.0% | 20.768 | 169.581 | 1.000 | 1.000 | 0.000 | 16.63x |
| `atropos` | 100.0% | 30.713 | 320.158 | 1.000 | 1.000 | 0.000 | 24.59x |
| `fastx_clipper` | 100.0% | 46.245 | 199.488 | 0.998 | 0.998 | 0.000 | 37.03x |
| `leehom` | 100.0% | 53.858 | 353.965 | 0.981 | 0.981 | 0.000 | 43.12x |
| `trimmomatic` | 0.0% | n/a | n/a | 0.000 | 0.000 | 0.000 | n/a |

## Cohort Behavior

- For `fastp`, `modern_pe` samples ran at `15.025` seconds median versus `6.139` for `ancient_se`.
- The fastest backend `seqkit` and highest-retention backend `atropos` are different, which matters when choosing a default objective.

## Highest-Cost Samples

| Sample | Accession | Cohort | Total runtime (s) | Slowest tool | Slowest runtime (s) | Lowest-retention tool | Lowest base retention |
| --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0018` | `ERR769591` | `ancient_se` | 1730.354 | `fastx_clipper` | 539.998 | `alientrimmer` | 0.000 |
| `sample_0006` | `DRR001073` | `modern_se` | 1530.433 | `leehom` | 418.413 | `alientrimmer` | 0.000 |
| `sample_0003` | `DRR000550` | `modern_pe` | 1251.509 | `leehom` | 353.965 | `alientrimmer` | 0.000 |
| `sample_0001` | `DRR000093` | `modern_pe` | 1161.203 | `leehom` | 328.435 | `alientrimmer` | 0.000 |
| `sample_0002` | `DRR000095` | `modern_pe` | 1138.873 | `leehom` | 325.009 | `alientrimmer` | 0.000 |

## Interpretation

- This corpus benchmark is intentionally bank-free for trim adapters/polyX/contaminants so the governed fairness cohort stays comparable across all included backends.
- The strongest choice for production depends on whether we prefer latency, base retention, or quality uplift; this dossier exposes those tradeoffs instead of collapsing them into one score.

## Reproducibility

- `summary.json`, `sample_results.csv`, and the CSV analysis tables in this directory are generated artifacts from the same corpus run.
- `sample_results.csv` preserves one row per sample/tool execution for independent reanalysis.
