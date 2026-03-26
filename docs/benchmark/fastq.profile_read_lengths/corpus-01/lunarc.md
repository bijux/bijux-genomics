# `fastq.profile_read_lengths` benchmark on `corpus-01`

## What was run

This benchmark measures the governed `fastq.profile_read_lengths` stage across the curated `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `/home/bijan/bijux/corpus_01`
- Benchmark root: `/home/bijan/bijux/corpus_01/benchmarks/fastq.profile_read_lengths/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `seqkit_stats`
- Length-profile contract: report_only=`True`, mutates_fastq=`False`, may_change_read_count=`False`, histogram_bins=`100`
- Governed artifacts per sample/tool: `profile_read_lengths_report.json`, `length_distribution.tsv`, and `length_distribution.json`.
- Execution profile: one benchmark sample at a time, one worker, governed thread budget

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `seqkit_stats` ran at `p50=1.603s` with median mean read length `76.000` and median distinct-length support `7.000`.
- Runtime remains input-driven for `seqkit_stats`: `modern_pe` averages `6.456s` while `ancient_se` averages `2.694s`.
- Size-band spread remains visible for `seqkit_stats`: `under_500mb` averages `8.074s` versus `1.065s` on `under_100mb` inputs.
- Correctness stayed stable across all `20` tool-sample observations: `exit_code=0` on `20` rows, and every published row carried governed histogram artifacts plus valid length-distribution metrics.
- Histogram resolution stayed pinned at `100` bins, so cross-sample comparisons use one deterministic bucket budget.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median read count | Median mean length | Median max length | Median distinct lengths |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `seqkit_stats` | 100.0% | 4.036 | 1.603 | 10.231 | 11.426 | 1241122.500 | 76.000 | 76.000 | 7.000 |

## Cohort behavior

| Tool | Dimension | Cohort | Samples | Mean runtime (s) | Median runtime (s) | Median mean length | Median max length | Median distinct lengths |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `seqkit_stats` | `era_layout` | `ancient_pe` | 5 | 2.786 | 1.160 | 75.697 | 76.000 | 13.000 |
| `seqkit_stats` | `era_layout` | `ancient_se` | 5 | 2.694 | 1.264 | 80.354 | 185.000 | 156.000 |
| `seqkit_stats` | `era_layout` | `modern_pe` | 5 | 6.456 | 10.003 | 36.000 | 36.000 | 1.000 |
| `seqkit_stats` | `era_layout` | `modern_se` | 5 | 4.208 | 1.849 | 76.000 | 76.000 | 1.000 |
| `seqkit_stats` | `size_band` | `under_1000mb` | 1 | 11.426 | 11.426 | 101.000 | 101.000 | 1.000 |
| `seqkit_stats` | `size_band` | `under_100mb` | 12 | 1.065 | 1.080 | 76.000 | 99.500 | 85.000 |
| `seqkit_stats` | `size_band` | `under_500mb` | 7 | 8.074 | 8.927 | 76.000 | 76.000 | 1.000 |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Slowest runtime (s) | Distinct lengths | Max read length |
| --- | --- | --- | --- | --- | ---: | ---: | ---: |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 11.426 | 1.000 | 101.000 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 11.016 | 1.000 | 36.000 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 10.231 | 1.000 | 36.000 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 10.003 | 1.000 | 36.000 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 8.927 | 158.000 | 187.000 |
| `sample_0013` | `ERR15886310` | `ancient` | `pe` | `under_500mb` | 6.011 | 1.000 | 76.000 |
| `sample_0008` | `DRR001083` | `modern` | `se` | `under_500mb` | 5.400 | 1.000 | 140.000 |
| `sample_0012` | `ERR15886307` | `ancient` | `pe` | `under_500mb` | 4.926 | 1.000 | 76.000 |
| `sample_0017` | `ERR769590` | `ancient` | `se` | `under_100mb` | 2.003 | 158.000 | 187.000 |
| `sample_0007` | `DRR001076` | `modern` | `se` | `under_100mb` | 1.849 | 1.000 | 76.000 |

## Interpretation

- `fastq.profile_read_lengths` is a non-mutating length-profile stage, so runtime and deterministic histogram stability matter more than retention deltas.
- Because the current governed benchmark cohort is a single backend, this dossier acts as a corpus-wide stability baseline for future regressions and future backend additions.
- Artifact integrity matters as much as the metrics here: without the governed TSV and JSON histogram outputs, downstream comparisons lose their canonical surface.
- `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv` are published beside this briefing for reproducible downstream analysis.
