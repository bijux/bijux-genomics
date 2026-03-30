# `fastq.extract_umis` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.extract_umis` stage across the paired subset of the `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

## Executive Summary

- `1` governed UMI extractor was benchmarked across `10` paired samples (`10/10` zero-exit tool-sample observations).
- Median runtime: `umi_tools` at `56.993` seconds.
- Mean reads-with-UMI fraction: `umi_tools` at `1.000`.

## Run Contract

- Platform: `lunarc-apptainer`
- Stage: `fastq.extract_umis`
- Scenario: `umi_extraction_fairness`
- Tools: `umi_tools`
- umi_pattern: `NNNNNNNN`
- allow_missing_umi_headers: `True`

## Tool Ranking

| Tool | Median runtime (s) | p90 runtime (s) | Median read retention | Mean reads with UMI | Mean reads with UMI fraction | Slowdown vs fastest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `umi_tools` | 56.993 | 189.663 | 1.000 | 5104575.400 | 1.000 | x1.00 |

## Cohort Behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median read retention | Mean reads with UMI fraction | Samples |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `umi_tools` | `ancient_pe` | 52.509 | 23.211 | 1.000 | 1.000 | 5 |
| `umi_tools` | `modern_pe` | 115.518 | 177.007 | 1.000 | 1.000 | 5 |
| `umi_tools` | `under_100mb` | 12.524 | 9.568 | 1.000 | 1.000 | 5 |
| `umi_tools` | `under_500mb` | 155.502 | 177.007 | 1.000 | 1.000 | 5 |

## Highest-Cost Samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Reads-with-UMI fraction |
| --- | --- | --- | --- | --- | ---: | --- | ---: | ---: |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 199.688 | `umi_tools` | 199.688 | 1.000 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 189.663 | `umi_tools` | 189.663 | 1.000 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 177.007 | `umi_tools` | 177.007 | 1.000 |
| `sample_0013` | `ERR15886310` | `ancient` | `pe` | `under_500mb` | 120.378 | `umi_tools` | 120.378 | 1.000 |
| `sample_0012` | `ERR15886307` | `ancient` | `pe` | `under_500mb` | 90.775 | `umi_tools` | 90.775 | 1.000 |

## Interpretation

- This paired-only briefing keeps the UMI pattern explicit so later barcode-policy changes cannot masquerade as benchmark regressions.
- Missing-header bypass is recorded in the run contract because `corpus-01` is a human DNA cohort rather than a native UMI corpus.
- The per-sample CSV artifacts make it easy to inspect whether runtime outliers coincide with weaker read retention or weaker UMI detection.
