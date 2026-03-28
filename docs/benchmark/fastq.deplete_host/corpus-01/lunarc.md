# `fastq.deplete_host` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.deplete_host` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

## Executive Summary

- `1` governed host-depletion backends were benchmarked across `20` samples (`20/20` zero-exit tool-sample observations).
- Fastest median runtime: `bowtie2` at `19.256` seconds.
- Highest mean host fraction removed: `bowtie2` at `0.646`.

## Run Contract

- Platform: `lunarc-apptainer`
- Stage: `fastq.deplete_host`
- Scenario: `host_depletion_fairness`
- Tools: `bowtie2`
- reference_index_digest: `639f1934a7933edae38d8bf42bf9f6cc43560e24686c53b0e94e27f9f6691831`
- reference_index_lineage_digest: `4680af76346224d129ceba82cb055f78b9936aa8bffd5e405b2b34acb04df872`
- reference_catalog_id: `host_reference`
- reference_index_backend: `bowtie2_build`
- host_identity_threshold: `0.95`
- retain_unmapped_only: `True`

## Tool Ranking

| Tool | Median runtime (s) | p90 runtime (s) | Median read retention | Median base retention | Mean host fraction removed | Mean reads removed | Slowdown vs fastest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `bowtie2` | 19.256 | 283.118 | 0.224 | 0.246 | 0.646 | 2383476.200 | x1.00 |

## Cohort Behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median read retention | Median base retention | Mean host fraction removed | Samples |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `bowtie2` | `ancient_pe` | 21.317 | 9.726 | 0.975 | 0.975 | 0.028 | 5 |
| `bowtie2` | `ancient_se` | 47.962 | 17.329 | 0.002 | 0.001 | 0.996 | 5 |
| `bowtie2` | `modern_pe` | 244.598 | 283.118 | 0.316 | 0.341 | 0.677 | 5 |
| `bowtie2` | `modern_se` | 49.117 | 21.183 | 0.042 | 0.042 | 0.884 | 5 |
| `bowtie2` | `under_1000mb` | 140.510 | 140.510 | 0.132 | 0.132 | 0.868 | 1 |
| `bowtie2` | `under_100mb` | 12.552 | 10.731 | 0.041 | 0.041 | 0.698 | 12 |
| `bowtie2` | `under_500mb` | 217.691 | 172.516 | 0.418 | 0.418 | 0.526 | 7 |

## Highest-Cost Samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest depletion tool | Host fraction removed |
| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 470.036 | `bowtie2` | 470.036 | `bowtie2` | 0.565 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 457.540 | `bowtie2` | 457.540 | `bowtie2` | 0.582 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 283.118 | `bowtie2` | 283.118 | `bowtie2` | 0.835 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 172.516 | `bowtie2` | 172.516 | `bowtie2` | 0.998 |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 140.510 | `bowtie2` | 140.510 | `bowtie2` | 0.868 |

## Interpretation

- Because the cohort itself is human DNA, this benchmark is intentionally unforgiving: high removal can indicate aggressive false-positive behavior rather than success.
- The published CSV artifacts keep index lineage and per-sample depletion outcomes explicit so later reruns can audit host-reference drift instead of relying on narrative summaries.
