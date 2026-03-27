# `fastq.normalize_primers` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.normalize_primers` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

## Executive Summary

- `1` governed primer-normalization backends were benchmarked across `20` samples (`20/20` zero-exit tool-sample observations).
- Fastest median runtime: `cutadapt` at `9.972` seconds.
- Highest mean primer-trimmed fraction: `cutadapt` at `0.000`.
- Highest median forward-orientation fraction: `cutadapt` at `0.000`.

## Run Contract

- Platform: `lunarc-apptainer`
- Stage: `fastq.normalize_primers`
- Scenario: `primer_normalization_fairness`
- Tools: `cutadapt`
- primer_set_id: `16S_universal_v1`
- orientation_policy: `normalize_to_forward_primer`
- max_mismatch_rate: `0.1`
- min_overlap_bp: `10`
- strict_5p_anchor: `True`
- allow_iupac_codes: `True`

## Tool Ranking

| Tool | Median runtime (s) | p90 runtime (s) | Median read retention | Mean primer-trimmed fraction | Median forward-orientation fraction | Slowdown vs fastest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `cutadapt` | 9.972 | 77.686 | 1.000 | 0.000 | 0.000 | x1.00 |

## Cohort Behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median read retention | Mean primer-trimmed fraction | Median forward-orientation fraction | Samples |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `cutadapt` | `ancient_pe` | 20.383 | 9.421 | 1.000 | 0.000 | 0.000 | 5 |
| `cutadapt` | `ancient_se` | 21.364 | 8.519 | 1.000 | 0.000 | 0.000 | 5 |
| `cutadapt` | `modern_pe` | 48.877 | 77.686 | 1.000 | 0.000 | 0.000 | 5 |
| `cutadapt` | `modern_se` | 26.612 | 10.522 | 1.000 | 0.000 | 0.000 | 5 |
| `cutadapt` | `under_1000mb` | 76.905 | 76.905 | 1.000 | 0.000 | 0.000 | 1 |
| `cutadapt` | `under_100mb` | 6.365 | 6.362 | 1.000 | 0.000 | 0.000 | 12 |
| `cutadapt` | `under_500mb` | 61.843 | 76.046 | 1.000 | 0.000 | 0.000 | 7 |

## Highest-Cost Samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest trim tool | Primer-trimmed fraction |
| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 84.758 | `cutadapt` | 84.758 | `cutadapt` | 0.000 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 78.639 | `cutadapt` | 78.639 | `cutadapt` | 0.000 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 77.686 | `cutadapt` | 77.686 | `cutadapt` | 0.000 |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 76.905 | `cutadapt` | 76.905 | `cutadapt` | 0.000 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 76.046 | `cutadapt` | 76.046 | `cutadapt` | 0.000 |

## Interpretation

- Because `corpus-01` is not an amplicon challenge set, non-zero primer trimming here should be interpreted as false-positive behavior unless independently justified by sequence evidence.
- The published CSV artifacts keep the governed primer policy and per-sample outcomes explicit so future reruns can audit drift instead of relying on narrative summaries alone.
