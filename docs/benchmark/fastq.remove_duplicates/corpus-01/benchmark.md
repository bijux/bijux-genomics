# `fastq.remove_duplicates` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.remove_duplicates` stage across the paired subset of the corpus-01 human DNA cohort on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `<REMOTE_CORPUS_ROOT>`
- Benchmark root: `<LOCAL_CACHE_RESULTS_ROOT>/corpus_01/fastq.remove_duplicates/lunarc`
- Input balance: `5` ancient, `5` modern, paired-end only
- Tool set: `clumpify, fastuniq`
- dedup_mode: `exact`
- keep_order: `True`

## Executive summary

- Every tool completed successfully on all `10` paired samples; stage-level sample failures were `0`.
- `fastest` median runtime is `clumpify` at `21.292s`; the slowest is `fastuniq` at `32.363s`.
- The strongest duplicate removal is `clumpify` with `1194672.1` duplicate reads removed on average.
- The highest median deduplication rate comes from `clumpify` at `0.088`.
- Correctness remained stable across `20` tool-sample observations: `20` rows finished with `exit_code=0`.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median dedup rate | Mean duplicate reads | Median slowdown |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `clumpify` | 100.0% | 50.881 | 21.292 | 140.544 | 190.810 | 0.088 | 1194672.1 | x1.00 |
| `fastuniq` | 100.0% | 47.255 | 32.363 | 106.903 | 142.621 | 0.061 | 1064557.7 | x1.52 |

## Cohort behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median dedup rate | Mean duplicate reads | Samples |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `clumpify` | `ancient_pe` | 70.956 | 11.152 | 0.018 | 50536.8 | 5 |
| `clumpify` | `modern_pe` | 30.806 | 31.432 | 0.145 | 2338807.4 | 5 |
| `clumpify` | `under_100mb` | 5.985 | 4.895 | 0.030 | 37427.0 | 5 |
| `clumpify` | `under_500mb` | 95.777 | 84.486 | 0.145 | 2351917.2 | 5 |
| `fastuniq` | `ancient_pe` | 45.212 | 17.045 | 0.010 | 36116.8 | 5 |
| `fastuniq` | `modern_pe` | 49.298 | 47.680 | 0.101 | 2092998.6 | 5 |
| `fastuniq` | `under_100mb` | 8.448 | 6.890 | 0.021 | 29385.8 | 5 |
| `fastuniq` | `under_500mb` | 86.062 | 83.961 | 0.101 | 2099729.6 | 5 |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest dedup tool | Duplicate reads removed |
| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |
| `sample_0013` | `ERR15886310` | `ancient` | `pe` | `under_500mb` | 297.713 | `clumpify` | 190.810 | `clumpify` | 52438.0 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 227.107 | `fastuniq` | 142.621 | `clumpify` | 974012.0 |
| `sample_0012` | `ERR15886307` | `ancient` | `pe` | `under_500mb` | 224.505 | `clumpify` | 140.544 | `clumpify` | 14922.0 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 80.759 | `fastuniq` | 49.145 | `clumpify` | 5352342.0 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 79.112 | `fastuniq` | 47.680 | `clumpify` | 5365872.0 |

## Interpretation

- This dossier makes the main tradeoff explicit: some backends remove more duplicates, while others preserve pair order or finish faster under the same governed contract.
- On corpus-01, this stage is benchmarking a real paired-end deduplication workload rather than a synthetic no-op path, so it is directly useful for later preprocessing defaults.

## Reproducibility

- Primary machine-readable artifacts beside this report: `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`.
