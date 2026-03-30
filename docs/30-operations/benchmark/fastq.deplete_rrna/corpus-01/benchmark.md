# `fastq.deplete_rrna` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.deplete_rrna` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

## Executive Summary

- `1` governed rRNA-depletion backends were benchmarked across `20` samples (`20/20` zero-exit tool-sample observations).
- Fastest median runtime: `sortmerna` at `84.631` seconds.
- Highest mean rRNA fraction removed: `sortmerna` at `0.002`.

## Run Contract

- Platform: `lunarc-apptainer`
- Stage: `fastq.deplete_rrna`
- Scenario: `rrna_depletion_fairness`
- Tools: `sortmerna`
- rrna_bundle_id: `sortmerna_v4_3_default_db`
- rrna_bundle_digest: `sha256:50e80a1a2d1e8c4e2265d3d84c082f258166d4d8f628cd0956c04b15c5fc1a53`
- min_identity: `0.95`

## Tool Ranking

| Tool | Median runtime (s) | p90 runtime (s) | Median read retention | Median base retention | Mean rRNA fraction removed | Mean reads removed | Slowdown vs fastest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `sortmerna` | 84.631 | 455.106 | 1.000 | 1.000 | 0.002 | 9646.250 | x1.00 |

## Cohort Behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median read retention | Median base retention | Mean rRNA fraction removed | Samples |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `sortmerna` | `ancient_pe` | 175.144 | 67.974 | 1.000 | 1.000 | 0.000 | 5 |
| `sortmerna` | `ancient_se` | 137.138 | 70.626 | 1.000 | 1.000 | 0.000 | 5 |
| `sortmerna` | `modern_pe` | 1237.501 | 416.483 | 1.000 | 1.000 | 0.003 | 5 |
| `sortmerna` | `modern_se` | 330.350 | 107.535 | 0.991 | 0.991 | 0.006 | 5 |
| `sortmerna` | `under_1000mb` | 1008.114 | 1008.114 | 1.000 | 1.000 | 0.000 | 1 |
| `sortmerna` | `under_100mb` | 55.571 | 53.138 | 1.000 | 1.000 | 0.002 | 12 |
| `sortmerna` | `under_500mb` | 1103.672 | 416.483 | 1.000 | 1.000 | 0.002 | 7 |

## Highest-Cost Samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest depletion tool | rRNA fraction removed |
| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 5290.197 | `sortmerna` | 5290.197 | `sortmerna` | 0.000 |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 1008.114 | `sortmerna` | 1008.114 | `sortmerna` | 0.000 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 455.106 | `sortmerna` | 455.106 | `sortmerna` | 0.000 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 429.149 | `sortmerna` | 429.149 | `sortmerna` | 0.007 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 416.483 | `sortmerna` | 416.483 | `sortmerna` | 0.007 |

## Interpretation

- Because `corpus-01` is not an rRNA-rich challenge set, any substantial depletion here should be treated as aggressive false-positive behavior unless independently justified.
- The published CSV artifacts keep bundle identity, digest, and per-sample depletion outcomes explicit so later reruns can audit reference drift instead of relying on narrative summaries.
