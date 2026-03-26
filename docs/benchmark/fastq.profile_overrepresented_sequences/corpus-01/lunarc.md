# `fastq.profile_overrepresented_sequences` benchmark on `corpus-01`

## What was run

This benchmark measures the governed `fastq.profile_overrepresented_sequences` stage across the curated `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `/home/bijan/bijux/corpus_01`
- Benchmark root: `/home/bijan/bijux/results/corpus_01/fastq.profile_overrepresented_sequences/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `fastq_scan, fastqc, seqkit`
- Sequence-profile contract: report_only=`True`, mutates_fastq=`False`, may_change_read_count=`False`, top_k=`50`
- Governed artifacts per sample/tool: `overrepresented_sequences.tsv`, `overrepresented_sequences.json`, and `overrepresented_report.json`.
- Execution profile: one benchmark sample at a time, one worker, governed thread budget

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `fastq_scan` ran at `p50=1.255s` with median sequence count `50.000` and median dominant-sequence fraction `0.002`.
- Runtime remains input-driven for `fastq_scan`: `modern_pe` averages `2.475s` while `ancient_se` averages `2.131s`.
- Size-band spread remains visible for `fastq_scan`: `under_500mb` averages `3.947s` versus `0.754s` on `under_100mb` inputs.
- Correctness stayed stable across all `60` tool-sample observations: `exit_code=0` on `60` rows, and every published row carried governed ranked-sequence artifacts with valid fraction bounds.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median sequence count | Median flagged sequences | Median top fraction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `fastq_scan` | 100.0% | 2.275 | 1.255 | 4.365 | 8.826 | 50.000 | 0.000 | 0.002 |
| `fastqc` | 100.0% | 15.532 | 7.087 | 35.560 | 41.572 | 50.000 | 0.000 | 0.002 |
| `seqkit` | 100.0% | 5.108 | 1.979 | 13.162 | 14.577 | 50.000 | 0.000 | 0.002 |

## Cohort behavior

| Tool | Dimension | Cohort | Samples | Mean runtime (s) | Median runtime (s) | Median sequence count | Median flagged sequences | Median top fraction |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fastq_scan` | `era_layout` | `ancient_pe` | 5 | 1.199 | 0.569 | 50.000 | 0.000 | 0.000 |
| `fastq_scan` | `era_layout` | `ancient_se` | 5 | 2.131 | 1.003 | 50.000 | 0.000 | 0.000 |
| `fastq_scan` | `era_layout` | `modern_pe` | 5 | 2.475 | 3.775 | 50.000 | 0.000 | 0.010 |
| `fastq_scan` | `era_layout` | `modern_se` | 5 | 3.295 | 1.449 | 50.000 | 0.000 | 0.009 |
| `fastqc` | `era_layout` | `ancient_pe` | 5 | 11.985 | 6.588 | 50.000 | 0.000 | 0.000 |
| `fastqc` | `era_layout` | `ancient_se` | 5 | 12.183 | 6.589 | 50.000 | 0.000 | 0.000 |
| `fastqc` | `era_layout` | `modern_pe` | 5 | 21.977 | 31.589 | 50.000 | 0.000 | 0.010 |
| `fastqc` | `era_layout` | `modern_se` | 5 | 15.984 | 7.585 | 50.000 | 0.000 | 0.009 |
| `seqkit` | `era_layout` | `ancient_pe` | 5 | 3.535 | 1.577 | 50.000 | 0.000 | 0.000 |
| `seqkit` | `era_layout` | `ancient_se` | 5 | 3.428 | 1.608 | 50.000 | 0.000 | 0.000 |
| `seqkit` | `era_layout` | `modern_pe` | 5 | 8.372 | 13.027 | 50.000 | 0.000 | 0.010 |
| `seqkit` | `era_layout` | `modern_se` | 5 | 5.098 | 2.260 | 50.000 | 0.000 | 0.009 |
| `fastq_scan` | `size_band` | `under_1000mb` | 1 | 8.826 | 8.826 | 50.000 | 0.000 | 0.004 |
| `fastq_scan` | `size_band` | `under_100mb` | 12 | 0.754 | 0.602 | 50.000 | 0.000 | 0.001 |
| `fastq_scan` | `size_band` | `under_500mb` | 7 | 3.947 | 3.853 | 50.000 | 0.000 | 0.003 |
| `fastqc` | `size_band` | `under_1000mb` | 1 | 41.572 | 41.572 | 50.000 | 0.000 | 0.004 |
| `fastqc` | `size_band` | `under_100mb` | 12 | 6.086 | 6.093 | 50.000 | 0.000 | 0.001 |
| `fastqc` | `size_band` | `under_500mb` | 7 | 28.006 | 31.589 | 50.000 | 0.000 | 0.003 |
| `seqkit` | `size_band` | `under_1000mb` | 1 | 13.746 | 13.746 | 50.000 | 0.000 | 0.004 |
| `seqkit` | `size_band` | `under_100mb` | 12 | 1.309 | 1.316 | 50.000 | 0.000 | 0.001 |
| `seqkit` | `size_band` | `under_500mb` | 7 | 10.388 | 11.518 | 50.000 | 0.000 | 0.003 |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Slowest runtime (s) | Dominant fraction | Flagged sequences |
| --- | --- | --- | --- | --- | ---: | ---: | ---: |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 41.572 | 0.004 | 0.000 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 35.566 | 0.019 | 1.000 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 35.560 | 0.000 | 0.000 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 32.565 | 0.011 | 2.000 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 31.589 | 0.010 | 0.000 |
| `sample_0013` | `ERR15886310` | `ancient` | `pe` | `under_500mb` | 21.578 | 0.000 | 0.000 |
| `sample_0008` | `DRR001083` | `modern` | `se` | `under_500mb` | 19.606 | 0.003 | 0.000 |
| `sample_0012` | `ERR15886307` | `ancient` | `pe` | `under_500mb` | 19.574 | 0.000 | 0.000 |
| `sample_0017` | `ERR769590` | `ancient` | `se` | `under_100mb` | 9.575 | 0.000 | 0.000 |
| `sample_0007` | `DRR001076` | `modern` | `se` | `under_100mb` | 7.585 | 0.009 | 0.000 |

## Interpretation

- `fastq.profile_overrepresented_sequences` is a non-mutating diagnostic stage, so runtime and ranked-sequence stability matter more than retention deltas.
- Because the governed benchmark cohort spans multiple observer backends, this dossier is meant to show both throughput and whether the normalized overrepresented-sequence summaries stay comparable across tools.
- `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv` are published beside this briefing for reproducible downstream analysis.
