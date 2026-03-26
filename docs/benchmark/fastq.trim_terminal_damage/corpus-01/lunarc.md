# `fastq.trim_terminal_damage` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.trim_terminal_damage` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

## Executive Summary

- `3` governed terminal-damage backends were benchmarked across `20` samples (`60/60` zero-exit tool-sample observations).
- Fastest median runtime: `adapterremoval` at `3.275` seconds.
- Slowest median runtime: `cutadapt` at `7.717` seconds.
- Terminal asymmetry reduction is not published for this corpus run, so backend ranking falls back to runtime and base-retention evidence.

## Run Contract

- Platform: `lunarc-apptainer`
- Stage: `fastq.trim_terminal_damage`
- Scenario: `terminal_damage_fairness`
- Tools: `adapterremoval, cutadapt, seqkit`
- damage_mode: `ancient`
- execution_policy: `explicit_terminal_trim`
- trim_5p_bases: `2`
- trim_3p_bases: `2`

## Tool Ranking

| Tool | Median runtime (s) | p90 runtime (s) | Median base retention | Mean asymmetry reduction | Mean Q delta | Slowdown vs fastest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `adapterremoval` | 3.275 | 24.471 | 0.942 | n/a | 0.000 | 1.00x |
| `seqkit` | 3.799 | 24.029 | 0.947 | n/a | 0.000 | 1.16x |
| `cutadapt` | 7.717 | 57.259 | 0.952 | n/a | 0.000 | 2.36x |

## Cohort Behavior

- Ancient and modern samples stay in the same benchmark corpus here so modern data act as a negative-control cohort for damage-aware trimming behavior.
- The best runtime backend and strongest damage-reduction backend may differ when asymmetry evidence is available, so this stage should not be judged on latency alone.

## Highest-Cost Samples

| Sample | Accession | Cohort | Total runtime (s) | Slowest tool | Slowest runtime (s) | Strongest damage tool | Asymmetry reduction |
| --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0003` | `DRR000550` | `modern_pe` | 113.948 | `cutadapt` | 63.156 | `n/a` | n/a |
| `sample_0006` | `DRR001073` | `modern_se` | 107.725 | `cutadapt` | 50.851 | `n/a` | n/a |
| `sample_0001` | `DRR000093` | `modern_pe` | 104.040 | `cutadapt` | 57.502 | `n/a` | n/a |
| `sample_0002` | `DRR000095` | `modern_pe` | 103.982 | `cutadapt` | 57.259 | `n/a` | n/a |
| `sample_0018` | `ERR769591` | `ancient_se` | 98.057 | `cutadapt` | 48.683 | `n/a` | n/a |

## Reproducibility

- `summary.json`, `sample_results.csv`, and the CSV analysis tables in this directory are generated from one governed run manifest.
- The renderer rejects dry-run manifests and mixed policy rows, so publication is tied to a real executed benchmark surface.
