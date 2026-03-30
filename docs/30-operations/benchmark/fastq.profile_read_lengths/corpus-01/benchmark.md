# `fastq.profile_read_lengths` benchmark on `corpus-01`

## What was run

This benchmark measures the governed `fastq.profile_read_lengths` stage across the curated `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `<REMOTE_CORPUS_ROOT>`
- Benchmark root: `<LOCAL_CACHE_RESULTS_ROOT>/corpus_01/fastq.profile_read_lengths/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `seqkit_stats`
- Length-profile contract: report_only=`True`, mutates_fastq=`False`, may_change_read_count=`False`, histogram_bins=`100`
- Governed artifacts per sample/tool: `profile_read_lengths_report.json`, `length_distribution.tsv`, and `length_distribution.json`.
- Execution profile: one benchmark sample at a time, one worker, governed thread budget

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `seqkit_stats` ran at `p50=1.646s` with median mean read length `76` and median distinct-length support `7`.
- Runtime remains input-driven for `seqkit_stats`: `modern_pe` averages `6.370s` while `ancient_se` averages `2.890s`.
- Size-band spread remains visible for `seqkit_stats`: `under_500mb` averages `8.134s` versus `1.084s` on `under_100mb` inputs.
- Correctness stayed stable across all `20` tool-sample observations: `exit_code=0` on `20` rows, and every published row carried governed histogram artifacts plus valid length-distribution metrics.
- Histogram resolution stayed pinned at `100` bins, so cross-sample comparisons use one deterministic bucket budget.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median read count | Median mean length | Median max length | Median distinct lengths |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `seqkit_stats` | 100.0% | 4.071 | 1.646 | 9.917 | 11.478 | 1241122.500 | 76 | 76 | 7 |

## Cohort behavior

| Tool | Dimension | Cohort | Samples | Mean runtime (s) | Median runtime (s) | Median mean length | Median max length | Median distinct lengths |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `seqkit_stats` | `era_layout` | `ancient_pe` | 5 | 2.769 | 1.189 | 75.697 | 76 | 13 |
| `seqkit_stats` | `era_layout` | `ancient_se` | 5 | 2.890 | 1.277 | 80.354 | 185 | 156 |
| `seqkit_stats` | `era_layout` | `modern_pe` | 5 | 6.370 | 9.892 | 36 | 36 | 1 |
| `seqkit_stats` | `era_layout` | `modern_se` | 5 | 4.255 | 1.877 | 76 | 76 | 1 |
| `seqkit_stats` | `size_band` | `under_1000mb` | 1 | 11.478 | 11.478 | 101 | 101 | 1 |
| `seqkit_stats` | `size_band` | `under_100mb` | 12 | 1.084 | 1.104 | 76 | 99.500 | 85 |
| `seqkit_stats` | `size_band` | `under_500mb` | 7 | 8.134 | 9.892 | 76 | 76 | 1 |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Slowest runtime (s) | Distinct lengths | Max read length |
| --- | --- | --- | --- | --- | ---: | ---: | ---: |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 11.478 | 1 | 101 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 10.964 | 1 | 36 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 9.917 | 1 | 36 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 9.914 | 158 | 187 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 9.892 | 1 | 36 |
| `sample_0013` | `ERR15886310` | `ancient` | `pe` | `under_500mb` | 5.987 | 1 | 76 |
| `sample_0008` | `DRR001083` | `modern` | `se` | `under_500mb` | 5.459 | 1 | 140 |
| `sample_0012` | `ERR15886307` | `ancient` | `pe` | `under_500mb` | 4.804 | 1 | 76 |
| `sample_0017` | `ERR769590` | `ancient` | `se` | `under_100mb` | 1.945 | 158 | 187 |
| `sample_0007` | `DRR001076` | `modern` | `se` | `under_100mb` | 1.877 | 1 | 76 |

## Interpretation

- `fastq.profile_read_lengths` is a non-mutating length-profile stage, so runtime and deterministic histogram stability matter more than retention deltas.
- Because the current governed benchmark cohort is a single backend, this dossier acts as a corpus-wide stability baseline for future regressions and future backend additions.
- Artifact integrity matters as much as the metrics here: without the governed TSV and JSON histogram outputs, downstream comparisons lose their canonical surface.
- `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv` are published beside this briefing for reproducible downstream analysis.
