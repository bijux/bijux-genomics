# `fastq.profile_overrepresented_sequences` benchmark on `corpus-01`

## What was run

This benchmark measures the governed `fastq.profile_overrepresented_sequences` stage across the curated `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `<REMOTE_CORPUS_ROOT>`
- Benchmark root: `<LOCAL_CACHE_RESULTS_ROOT>/corpus_01/fastq.profile_overrepresented_sequences/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `fastq_scan, fastqc, seqkit`
- Sequence-profile contract: report_only=`True`, mutates_fastq=`False`, may_change_read_count=`False`, top_k=`50`
- Governed artifacts per sample/tool: `overrepresented_sequences.tsv`, `overrepresented_sequences.json`, and `overrepresented_report.json`.
- Execution profile: one benchmark sample at a time, one worker, governed thread budget

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `fastq_scan` ran at `p50=1.259s` with median sequence count `50` and median dominant-sequence fraction `0.002`.
- Runtime remains input-driven for `fastq_scan`: `modern_pe` averages `2.454s` while `ancient_se` averages `2.251s`.
- Size-band spread remains visible for `fastq_scan`: `under_500mb` averages `4.015s` versus `0.784s` on `under_100mb` inputs.
- Correctness stayed stable across all `60` tool-sample observations: `exit_code=0` on `60` rows, and every published row carried governed ranked-sequence artifacts with valid fraction bounds.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median sequence count | Median flagged sequences | Median top fraction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `fastq_scan` | 100.0% | 2.347 | 1.259 | 4.390 | 9.419 | 50 | 0 | 0.002 |
| `fastqc` | 100.0% | 15.601 | 7.089 | 33.586 | 42.574 | 50 | 0 | 0.002 |
| `seqkit` | 100.0% | 5.122 | 1.961 | 13.421 | 14.816 | 50 | 0 | 0.002 |

## Cohort behavior

| Tool | Dimension | Cohort | Samples | Mean runtime (s) | Median runtime (s) | Median sequence count | Median flagged sequences | Median top fraction |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fastq_scan` | `era_layout` | `ancient_pe` | 5 | 1.268 | 0.642 | 50 | 0 | 0.000 |
| `fastq_scan` | `era_layout` | `ancient_se` | 5 | 2.251 | 1.116 | 50 | 0 | 0.000 |
| `fastq_scan` | `era_layout` | `modern_pe` | 5 | 2.454 | 3.582 | 50 | 0 | 0.010 |
| `fastq_scan` | `era_layout` | `modern_se` | 5 | 3.413 | 1.402 | 50 | 0 | 0.009 |
| `fastq_scan` | `size_band` | `under_1000mb` | 1 | 9.419 | 9.419 | 50 | 0 | 0.004 |
| `fastq_scan` | `size_band` | `under_100mb` | 12 | 0.784 | 0.635 | 50 | 0 | 0.001 |
| `fastq_scan` | `size_band` | `under_500mb` | 7 | 4.015 | 3.719 | 50 | 0 | 0.003 |
| `fastqc` | `era_layout` | `ancient_pe` | 5 | 12.211 | 6.623 | 50 | 0 | 0.000 |
| `fastqc` | `era_layout` | `ancient_se` | 5 | 12.613 | 6.622 | 50 | 0 | 0.000 |
| `fastqc` | `era_layout` | `modern_pe` | 5 | 21.394 | 31.619 | 50 | 0 | 0.010 |
| `fastqc` | `era_layout` | `modern_se` | 5 | 16.187 | 7.556 | 50 | 0 | 0.009 |
| `fastqc` | `size_band` | `under_1000mb` | 1 | 42.574 | 42.574 | 50 | 0 | 0.004 |
| `fastqc` | `size_band` | `under_100mb` | 12 | 5.855 | 5.625 | 50 | 0 | 0.001 |
| `fastqc` | `size_band` | `under_500mb` | 7 | 28.456 | 31.619 | 50 | 0 | 0.003 |
| `seqkit` | `era_layout` | `ancient_pe` | 5 | 3.521 | 1.548 | 50 | 0 | 0.000 |
| `seqkit` | `era_layout` | `ancient_se` | 5 | 3.426 | 1.565 | 50 | 0 | 0.000 |
| `seqkit` | `era_layout` | `modern_pe` | 5 | 8.495 | 13.108 | 50 | 0 | 0.010 |
| `seqkit` | `era_layout` | `modern_se` | 5 | 5.045 | 2.219 | 50 | 0 | 0.009 |
| `seqkit` | `size_band` | `under_1000mb` | 1 | 13.730 | 13.730 | 50 | 0 | 0.004 |
| `seqkit` | `size_band` | `under_100mb` | 12 | 1.307 | 1.347 | 50 | 0 | 0.001 |
| `seqkit` | `size_band` | `under_500mb` | 7 | 10.431 | 11.570 | 50 | 0 | 0.003 |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Slowest runtime (s) | Dominant fraction | Flagged sequences |
| --- | --- | --- | --- | --- | ---: | ---: | ---: |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 42.574 | 0.004 | 0 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 38.611 | 0.000 | 0 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 33.586 | 0.019 | 1 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 32.599 | 0.010 | 0 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 31.619 | 0.011 | 2 |
| `sample_0013` | `ERR15886310` | `ancient` | `pe` | `under_500mb` | 23.665 | 0.000 | 0 |
| `sample_0008` | `DRR001083` | `modern` | `se` | `under_500mb` | 20.550 | 0.003 | 0 |
| `sample_0012` | `ERR15886307` | `ancient` | `pe` | `under_500mb` | 18.563 | 0.000 | 0 |
| `sample_0017` | `ERR769590` | `ancient` | `se` | `under_100mb` | 8.598 | 0.000 | 0 |
| `sample_0007` | `DRR001076` | `modern` | `se` | `under_100mb` | 7.556 | 0.009 | 0 |

## Interpretation

- `fastq.profile_overrepresented_sequences` is a non-mutating diagnostic stage, so runtime and ranked-sequence stability matter more than retention deltas.
- Because the governed benchmark cohort spans multiple observer backends, this dossier is meant to show both throughput and whether the normalized overrepresented-sequence summaries stay comparable across tools.
- `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv` are published beside this briefing for reproducible downstream analysis.
