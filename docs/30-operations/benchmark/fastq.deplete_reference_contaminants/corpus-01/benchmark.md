# `fastq.deplete_reference_contaminants` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.deplete_reference_contaminants` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

## Executive Summary

- `1` governed contaminant-depletion backends were benchmarked across `20` samples (`20/20` zero-exit tool-sample observations).
- Fastest median runtime: `bowtie2` at `18.534` seconds.
- Highest mean contaminant fraction removed: `bowtie2` at `0.000`.

## Run Contract

- Platform: `lunarc-apptainer`
- Stage: `fastq.deplete_reference_contaminants`
- Scenario: `contaminant_depletion_fairness`
- Tools: `bowtie2`
- reference_index_digest: `8f0f646e6e37ed01b2a33291e96c2a38eeedeb3ca6f8175b685bd6f98719fc8f`
- reference_catalog_id: `contaminant_reference`
- reference_index_backend: `bowtie2_build`
- decoy_mode: `phix_and_spikeins`

## Tool Ranking

| Tool | Median runtime (s) | p90 runtime (s) | Median read retention | Median base retention | Mean contaminant fraction removed | Mean reads removed | Slowdown vs fastest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `bowtie2` | 18.534 | 77.911 | 1.000 | 1.000 | 0.000 | 0.200 | x1.00 |

## Cohort Behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median read retention | Median base retention | Mean contaminant fraction removed | Samples |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `bowtie2` | `ancient_pe` | 20.168 | 7.427 | 1.000 | 1.000 | 0.000 | 5 |
| `bowtie2` | `ancient_se` | 33.682 | 15.615 | 1.000 | 1.000 | 0.000 | 5 |
| `bowtie2` | `modern_pe` | 38.023 | 58.581 | 1.000 | 1.000 | 0.000 | 5 |
| `bowtie2` | `modern_se` | 57.144 | 21.454 | 1.000 | 1.000 | 0.000 | 5 |
| `bowtie2` | `under_1000mb` | 162.160 | 162.160 | 1.000 | 1.000 | 0.000 | 1 |
| `bowtie2` | `under_100mb` | 9.968 | 7.853 | 1.000 | 1.000 | 0.000 | 12 |
| `bowtie2` | `under_500mb` | 66.186 | 61.720 | 1.000 | 1.000 | 0.000 | 7 |

## Highest-Cost Samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest depletion tool | Contaminant fraction removed |
| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 162.160 | `bowtie2` | 162.160 | `bowtie2` | 0.000 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 115.750 | `bowtie2` | 115.750 | `bowtie2` | 0.000 |
| `sample_0008` | `DRR001083` | `modern` | `se` | `under_500mb` | 77.911 | `bowtie2` | 77.911 | `bowtie2` | 0.000 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 65.814 | `bowtie2` | 65.814 | `bowtie2` | 0.000 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 61.720 | `bowtie2` | 61.720 | `bowtie2` | 0.000 |

## Interpretation

- Because this cohort is human DNA, substantial contaminant depletion usually signals aggressive technical over-removal unless independently justified by the reference set.
- The published CSV artifacts keep reference lineage and governed decoy policy explicit so later reruns can audit reference drift instead of relying on narrative summaries.
