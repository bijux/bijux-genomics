# `fastq.profile_reads` benchmark on `corpus-01`

## What was run

This benchmark measures the governed `fastq.profile_reads` stage across the curated `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `<REMOTE_CORPUS_ROOT>`
- Benchmark root: `<LOCAL_CACHE_RESULTS_ROOT>/corpus_01/fastq.profile_reads/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `seqkit_stats`
- Profile contract: report_only=`True`, mutates_fastq=`False`, may_change_read_count=`False`, histogram=`seqkit_fx2tab`
- Execution profile: one benchmark sample at a time, one worker, governed thread budget

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `seqkit_stats` ran at `p50=1.614s` with median mean Q `0` and median GC `46.540`.
- Runtime remains input-driven for `seqkit_stats`: `modern_pe` averages `6.442s` while `ancient_se` averages `2.772s`.
- Size-band spread remains visible for `seqkit_stats`: `under_500mb` averages `8.120s` versus `1.083s` on `under_100mb` inputs.
- Correctness stayed stable across all `20` tool-sample observations: `exit_code=0` on `20` rows, and every published row kept positive totals with non-empty histogram support.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median reads | Median bases | Median Q | Median GC | Median read length |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `seqkit_stats` | 100.0% | 4.091 | 1.614 | 10.027 | 11.984 | 1241122.500 | 78085583.500 | 0 | 46.540 | 76 |

## Cohort behavior

| Tool | Dimension | Cohort | Samples | Mean runtime (s) | Median runtime (s) | Median Q | Median GC | Median read length | Median histogram bins |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `seqkit_stats` | `era_layout` | `ancient_pe` | 5 | 2.758 | 1.171 | 0 | 56.085 | 75.697 | 13 |
| `seqkit_stats` | `era_layout` | `ancient_se` | 5 | 2.772 | 1.295 | 0 | 45.840 | 80.354 | 156 |
| `seqkit_stats` | `era_layout` | `modern_pe` | 5 | 6.442 | 9.948 | 0 | 53.365 | 36 | 1 |
| `seqkit_stats` | `era_layout` | `modern_se` | 5 | 4.392 | 1.864 | 0 | 45.150 | 76 | 1 |
| `seqkit_stats` | `size_band` | `under_1000mb` | 1 | 11.984 | 11.984 | 0 | 45.150 | 101 | 1 |
| `seqkit_stats` | `size_band` | `under_100mb` | 12 | 1.083 | 1.093 | 0 | 46.300 | 76 | 85 |
| `seqkit_stats` | `size_band` | `under_500mb` | 7 | 8.120 | 9.274 | 0 | 50.470 | 76 | 1 |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Slowest runtime (s) | Histogram bins | Max observed length |
| --- | --- | --- | --- | --- | ---: | ---: | ---: |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 11.984 | 1 | 101 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 11.120 | 1 | 36 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 10.027 | 1 | 36 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 9.948 | 1 | 36 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 9.274 | 158 | 187 |
| `sample_0013` | `ERR15886310` | `ancient` | `pe` | `under_500mb` | 5.975 | 1 | 76 |
| `sample_0008` | `DRR001083` | `modern` | `se` | `under_500mb` | 5.718 | 1 | 140 |
| `sample_0012` | `ERR15886307` | `ancient` | `pe` | `under_500mb` | 4.781 | 1 | 76 |
| `sample_0017` | `ERR769590` | `ancient` | `se` | `under_100mb` | 1.913 | 158 | 187 |
| `sample_0007` | `DRR001076` | `modern` | `se` | `under_100mb` | 1.864 | 1 | 76 |

## Interpretation

- `fastq.profile_reads` is a non-mutating profile stage, so runtime and normalized report stability matter more than retention deltas.
- Because the current governed benchmark cohort is a single backend, this dossier acts as a corpus-wide stability baseline for future regressions and future backend additions.
- `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv` are published beside this briefing for reproducible downstream analysis.
