# `fastq.trim_polyg_tails` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.trim_polyg_tails` stage across the full corpus-01 human DNA cohort on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `/home/bijan/bijux/corpus_01`
- Benchmark root: `/home/bijan/bijux/corpus_01/benchmarks/fastq.trim_polyg_tails/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `bbduk, fastp`
- Chemistry preset: `illumina_twocolor`
- Execution profile: one benchmark sample at a time, one worker, min_polyg_run `10`

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `fastp` is the fastest backend at `p50=4.038s`, while `bbduk` is slower at `p50=5.998s`.
- The median slowdown of `bbduk` relative to the fastest backend is `x1.00`.
- Mean polyG trimming per sample is `fastp=34937548.500` bases and `bbduk=101119.800` bases.
- Runtime pressure is carried by paired modern inputs: `fastp modern_pe` averages `10.919s` while `fastp modern_se` averages `10.072s`.
- Input size remains the main cost driver: `fastp` averages `18.682s` on `under_500mb` samples versus `3.958s` on `under_100mb` samples.
- Correctness stayed stable across all `40` tool-sample observations: `exit_code=0` on `40` rows, with positive polyG trimming observed on `37` rows.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median base retention | Mean bases trimmed | Mean Q delta | Median slowdown |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `bbduk` | 100.0% | 7.512 | 4.038 | 14.297 | 29.075 | 1.000 | 101119.800 | 0.000 | x1.00 |
| `fastp` | 100.0% | 9.972 | 5.998 | 21.166 | 24.124 | 0.920 | 34937548.500 | 0.000 | x1.49 |

## Cohort behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Mean bases trimmed | Median base retention | Mean Q delta | Samples |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `bbduk` | `ancient_pe` | 4.373 | 2.139 | 132541.800 | 1.000 | 0.000 | 5 |
| `bbduk` | `ancient_se` | 7.254 | 3.133 | 24.200 | 1.000 | 0.000 | 5 |
| `bbduk` | `modern_pe` | 7.611 | 11.493 | 263693.000 | 1.000 | 0.000 | 5 |
| `bbduk` | `modern_se` | 10.812 | 4.681 | 8220.200 | 1.000 | 0.000 | 5 |
| `fastp` | `ancient_pe` | 9.686 | 3.835 | 53193973.000 | 0.754 | 0.000 | 5 |
| `fastp` | `ancient_se` | 9.211 | 5.712 | 3809.800 | 1.000 | 0.000 | 5 |
| `fastp` | `modern_pe` | 10.919 | 15.585 | 18391759.000 | 0.919 | 0.000 | 5 |
| `fastp` | `modern_se` | 10.072 | 6.284 | 68160652.200 | 0.907 | 0.000 | 5 |

## Size-band behavior

| Tool | Size band | Mean runtime (s) | Mean bases trimmed | Median base retention | Samples |
| --- | --- | ---: | ---: | ---: | ---: |
| `bbduk` | `under_1000mb` | 29.075 | 24249.000 | 1.000 | 1 |
| `bbduk` | `under_100mb` | 2.524 | 3521.417 | 1.000 | 12 |
| `bbduk` | `under_500mb` | 12.983 | 279412.857 | 0.999 | 7 |
| `fastp` | `under_1000mb` | 21.166 | 241346917.000 | 0.683 | 1 |
| `fastp` | `under_100mb` | 3.958 | 3183724.167 | 0.925 | 12 |
| `fastp` | `under_500mb` | 18.682 | 59885623.286 | 0.919 | 7 |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest trim tool | Strongest trim bases |
| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 50.242 | `bbduk` | 29.075 | `fastp` | 241346917.000 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 48.406 | `bbduk` | 24.282 | `fastp` | 14765.000 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 32.583 | `fastp` | 19.780 | `fastp` | 39120920.000 |
| `sample_0013` | `ERR15886310` | `ancient` | `pe` | `under_500mb` | 31.167 | `fastp` | 21.831 | `fastp` | 144883652.000 |
| `sample_0008` | `DRR001083` | `modern` | `se` | `under_500mb` | 28.891 | `fastp` | 14.594 | `fastp` | 83810677.000 |

## Interpretation

- `fastp` is the lower-latency default for corpus-scale polyG cleanup, while `bbduk` trades more wall time for comparable retention.
- The benchmark is dominated by mid-size paired inputs, so the stage should be budgeted as a paired-end cost center rather than a single-end one.
- Positive trim counts across the corpus show that this stage is not acting as a pure no-op on corpus-01; the chosen chemistry preset is exercising real cleanup work.

## Reproducibility

- Primary machine-readable artifacts beside this report: `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`.
- Input cohort metadata is joined through the committed `corpus-01` spec and the materialized corpus manifest, so accession-to-sample identity remains stable across rerenders.
