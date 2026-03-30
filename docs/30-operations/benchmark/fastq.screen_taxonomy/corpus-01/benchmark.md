# `fastq.screen_taxonomy` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.screen_taxonomy` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

## Executive Summary

- `4` governed taxonomy classifiers were benchmarked across `20` samples (`80/80` zero-exit tool-sample observations).
- Fastest median runtime: `kraken2` at `1.392` seconds.
- Lowest mean contamination rate: `centrifuge` at `0.000`.

## Run Contract

- Platform: `lunarc-apptainer`
- Stage: `fastq.screen_taxonomy`
- Scenario: `screen_fairness`
- Tools: `centrifuge, kaiju, kraken2, krakenuniq`
- database_digest: `5bb5be0ced539ee0d1cd88d41f4f63781d8941d1964ef7410c1b16995795f7de`
- database_lineage_digest: `6c26e735250bd7438dfe1c2404e4dca6507c98aa0520a2bfd9024b4bebfd92a8`
- database_catalog_id: `taxonomy_reference`
- database_artifact_id: `taxonomy_db`
- database_namespace: `read_screening`
- database_scope: `read_screening`

## Tool Ranking

| Tool | Median runtime (s) | p90 runtime (s) | Mean contamination rate | Mean classified fraction | Mean unclassified fraction | Slowdown vs fastest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `centrifuge` | 4.016 | 28.875 | 0.000 | 0.000 | 0.000 | x2.89 |
| `kaiju` | 2.874 | 24.782 | 0.000 | 0.000 | 0.000 | x2.07 |
| `kraken2` | 1.392 | 8.923 | 0.000 | 0.000 | 1.000 | x1.00 |
| `krakenuniq` | 2.622 | 29.536 | 0.000 | 0.000 | 1.000 | x1.88 |

## Cohort Behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Mean contamination rate | Mean classified fraction | Samples |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `centrifuge` | `ancient_pe` | 9.234 | 3.821 | 0.000 | 0.000 | 5 |
| `centrifuge` | `ancient_se` | 7.847 | 3.189 | 0.000 | 0.000 | 5 |
| `centrifuge` | `modern_pe` | 17.888 | 25.637 | 0.000 | 0.000 | 5 |
| `centrifuge` | `modern_se` | 10.888 | 4.210 | 0.000 | 0.000 | 5 |
| `centrifuge` | `under_1000mb` | 30.248 | 30.248 | 0.000 | 0.000 | 1 |
| `centrifuge` | `under_100mb` | 2.599 | 2.486 | 0.000 | 0.000 | 12 |
| `centrifuge` | `under_500mb` | 23.977 | 25.637 | 0.000 | 0.000 | 7 |
| `kaiju` | `ancient_pe` | 6.381 | 2.499 | 0.000 | 0.000 | 5 |
| `kaiju` | `ancient_se` | 6.134 | 2.709 | 0.000 | 0.000 | 5 |
| `kaiju` | `modern_pe` | 14.881 | 22.003 | 0.000 | 0.000 | 5 |
| `kaiju` | `modern_se` | 9.555 | 3.040 | 0.000 | 0.000 | 5 |
| `kaiju` | `under_1000mb` | 26.600 | 26.600 | 0.000 | 0.000 | 1 |
| `kaiju` | `under_100mb` | 2.114 | 2.291 | 0.000 | 0.000 | 12 |
| `kaiju` | `under_500mb` | 18.969 | 20.503 | 0.000 | 0.000 | 7 |
| `kraken2` | `ancient_pe` | 2.772 | 1.101 | 0.000 | 0.000 | 5 |
| `kraken2` | `ancient_se` | 2.767 | 1.239 | 0.000 | 0.000 | 5 |
| `kraken2` | `modern_pe` | 5.404 | 8.288 | 0.000 | 0.000 | 5 |
| `kraken2` | `modern_se` | 3.899 | 1.544 | 0.000 | 0.000 | 5 |
| `kraken2` | `under_1000mb` | 10.621 | 10.621 | 0.000 | 0.000 | 1 |
| `kraken2` | `under_100mb` | 1.002 | 0.991 | 0.000 | 0.000 | 12 |
| `kraken2` | `under_500mb` | 7.366 | 8.288 | 0.000 | 0.000 | 7 |
| `krakenuniq` | `ancient_pe` | 8.761 | 3.864 | 0.000 | 0.000 | 5 |
| `krakenuniq` | `ancient_se` | 3.095 | 1.660 | 0.000 | 0.000 | 5 |
| `krakenuniq` | `modern_pe` | 23.604 | 29.536 | 0.000 | 0.000 | 5 |
| `krakenuniq` | `modern_se` | 4.643 | 1.486 | 0.000 | 0.000 | 5 |
| `krakenuniq` | `under_1000mb` | 13.525 | 13.525 | 0.000 | 0.000 | 1 |
| `krakenuniq` | `under_100mb` | 1.648 | 1.325 | 0.000 | 0.000 | 12 |
| `krakenuniq` | `under_500mb` | 23.887 | 18.552 | 0.000 | 0.000 | 7 |

## Highest-Cost Samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Highest-contamination tool | Contamination rate |
| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 120.676 | `krakenuniq` | 53.398 | `centrifuge` | 0.000 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 92.453 | `krakenuniq` | 33.358 | `centrifuge` | 0.000 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 88.703 | `krakenuniq` | 29.536 | `centrifuge` | 0.000 |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 80.994 | `centrifuge` | 30.248 | `centrifuge` | 0.000 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 66.741 | `centrifuge` | 27.507 | `centrifuge` | 0.000 |

## Interpretation

- Because this cohort is human DNA, apparent contamination mostly reflects classifier background behavior and database composition rather than true mixed-sample challenge signal.
- The published CSV artifacts keep database lineage and per-sample contamination summaries explicit so later reruns can audit taxonomy drift instead of relying on narrative summaries.
