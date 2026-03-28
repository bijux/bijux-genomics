# `fastq.filter_low_complexity` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.filter_low_complexity` stage across the full corpus-01 human DNA cohort on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `<REMOTE_CORPUS_ROOT>`
- Benchmark root: `<LOCAL_CACHE_RESULTS_ROOT>/corpus_01/fastq.filter_low_complexity/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `bbduk, prinseq`
- entropy_threshold: `0.55`
- polyx_threshold: `unset`

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `fastest` median runtime is `bbduk` at `3.468s`; the slowest is `prinseq` at `7.380s`.
- The most aggressive filter is `bbduk` with `245897.5` low-complexity reads removed on average.
- The highest median base retention comes from `bbduk` at `0.999`.
- Correctness remained stable across `40` tool-sample observations: `40` rows finished with `exit_code=0`.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median base retention | Median read retention | Mean removed reads | Mean Q delta | Median slowdown |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `bbduk` | 100.0% | 6.238 | 3.468 | 12.347 | 23.356 | 0.999 | 0.999 | 245897.5 | 0.000 | x1.00 |
| `prinseq` | 100.0% | 20.753 | 7.380 | 54.537 | 62.286 | 0.999 | 0.999 | 92116.8 | 0.000 | x2.13 |

## Cohort behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Mean removed reads | Median base retention | Median read retention | Mean Q delta | Samples |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `bbduk` | `ancient_pe` | 3.568 | 1.781 | 90168.0 | 0.999 | 0.999 | 0.000 | 5 |
| `bbduk` | `ancient_se` | 6.096 | 3.081 | 10321.4 | 0.997 | 0.995 | 0.000 | 5 |
| `bbduk` | `modern_pe` | 6.312 | 8.807 | 882125.2 | 0.902 | 0.902 | 0.000 | 5 |
| `bbduk` | `modern_se` | 8.977 | 3.854 | 975.6 | 1.000 | 1.000 | 0.000 | 5 |
| `prinseq` | `ancient_pe` | 14.398 | 5.774 | 4602.8 | 0.999 | 0.999 | 0.000 | 5 |
| `prinseq` | `ancient_se` | 17.294 | 6.649 | 3727.2 | 0.998 | 0.997 | 0.000 | 5 |
| `prinseq` | `modern_pe` | 30.306 | 42.413 | 358556.0 | 0.954 | 0.954 | 0.000 | 5 |
| `prinseq` | `modern_se` | 21.016 | 8.110 | 1581.2 | 1.000 | 1.000 | 0.000 | 5 |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest filter tool | Strongest filter reads |
| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 81.580 | `prinseq` | 62.286 | `bbduk` | 13723.0 |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 80.798 | `prinseq` | 57.442 | `prinseq` | 4727.0 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 64.781 | `prinseq` | 54.537 | `bbduk` | 1313192.0 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 61.365 | `prinseq` | 51.662 | `bbduk` | 1517662.0 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 51.221 | `prinseq` | 42.413 | `bbduk` | 1579686.0 |

## Interpretation

- This dossier makes the main tradeoff explicit: some backends remove more low-complexity reads, while others preserve more sequence and finish faster.
- On corpus-01, the stage behaves as a real filter rather than a no-op, so this is a meaningful runtime-versus-retention comparison for future preprocessing defaults.

## Reproducibility

- Primary machine-readable artifacts beside this report: `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`.
