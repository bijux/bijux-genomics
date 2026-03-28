# `fastq.merge_pairs` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.merge_pairs` stage across the paired-end human DNA subset of corpus-01 on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `<REMOTE_CORPUS_ROOT>`
- Benchmark root: `<LOCAL_CACHE_RESULTS_ROOT>/corpus_01/fastq.merge_pairs/lunarc`
- Input balance: `5` ancient paired-end and `5` modern paired-end samples
- Tool set: `adapterremoval, bbmerge, flash2, leehom, pear, vsearch`
- Fixed merge contract: overlap `auto`, min merged length `auto`, unmerged policy `emit_unmerged_pairs`
- Execution profile: one benchmark sample at a time, one worker, with sample-level outputs retained in the Lunarc results tree and mirrored locally.

## Executive summary

- Every tool completed successfully on all `10` paired samples; stage-level sample failures were `0`.
- `fastest p50 runtime` is `vsearch` at `10.166s`, while the slowest median backend is `leehom` at `85.728s`.
- `best p50 merge rate` is `adapterremoval` at `0.873`, with median base retention `0.325`.
- Runtime spread from fastest to slowest median backend is `x8.43`.
- Cohort pressure is higher for modern paired libraries: `adapterremoval modern_pe` averages `72.775s` versus `18.342s` on `ancient_pe`.
- All `60` tool-sample observations exited cleanly; zero-exit observations were `60`.

## Tool ranking

| Tool | p50 runtime (s) | p90 runtime (s) | Median merge rate | Median base retention | Mean merged reads |
| --- | ---: | ---: | ---: | ---: | ---: |
| `adapterremoval` | 19.119 | 116.751 | 0.873 | 0.325 | 1040046.900 |
| `bbmerge` | 55.430 | 107.385 | 0.567 | 0.263 | 425296.900 |
| `flash2` | 12.107 | 25.228 | 0.531 | 0.209 | 715652.200 |
| `leehom` | 85.728 | 342.872 | 0.402 | 0.143 | 637122.400 |
| `pear` | 72.144 | 132.494 | 0.211 | 0.123 | 245063.300 |
| `vsearch` | 10.166 | 28.240 | 0.121 | 0.070 | 187280.800 |

## Cohort behavior

| Tool | Cohort | Mean runtime (s) | Mean merge rate | Median base retention |
| --- | --- | ---: | ---: | ---: |
| `adapterremoval` | `ancient_pe` | 18.342 | 0.923 | 0.370 |
| `adapterremoval` | `modern_pe` | 72.775 | 0.474 | 0.133 |
| `bbmerge` | `ancient_pe` | 56.391 | 0.614 | 0.316 |
| `bbmerge` | `modern_pe` | 63.339 | 0.302 | 0.029 |
| `flash2` | `ancient_pe` | 10.719 | 0.728 | 0.356 |
| `flash2` | `modern_pe` | 15.748 | 0.308 | 0.123 |
| `leehom` | `ancient_pe` | 75.511 | 0.786 | 0.253 |
| `leehom` | `modern_pe` | 212.626 | 0.171 | 0.011 |
| `pear` | `ancient_pe` | 71.411 | 0.355 | 0.138 |
| `pear` | `modern_pe` | 77.212 | 0.388 | 0.042 |
| `vsearch` | `ancient_pe` | 9.328 | 0.331 | 0.114 |
| `vsearch` | `modern_pe` | 17.862 | 0.163 | 0.043 |

## Size-band behavior

| Tool | Size band | Mean runtime (s) | Mean merge rate | Mean merged reads |
| --- | --- | ---: | ---: | ---: |
| `adapterremoval` | `under_100mb` | 4.882 | 0.875 | 251230.600 |
| `adapterremoval` | `under_500mb` | 86.234 | 0.522 | 1828863.200 |

## Highest-cost samples

| Sample | Accession | Era | Size band | Total runtime (s) | Slowest tool | Best merge-rate tool |
| --- | --- | --- | --- | ---: | --- | --- |
| `sample_0003` | `DRR000550` | `modern` | `under_500mb` | 768.914 | `leehom` | `adapterremoval` |
| `sample_0001` | `DRR000093` | `modern` | `under_500mb` | 711.408 | `leehom` | `adapterremoval` |
| `sample_0002` | `DRR000095` | `modern` | `under_500mb` | 700.241 | `leehom` | `adapterremoval` |
| `sample_0013` | `ERR15886310` | `ancient` | `under_500mb` | 572.276 | `leehom` | `adapterremoval` |
| `sample_0012` | `ERR15886307` | `ancient` | `under_500mb` | 434.764 | `pear` | `adapterremoval` |

## Interpretation

- Merge-rate comparisons in this dossier are only valid because overlap threshold, minimum merged length, and unmerged-mate handling are fixed across the full cohort.
- Base-retention differences should be read together with merge rate: a backend can preserve bases while still collapsing fewer pairs.
- Ancient paired libraries remain important in this corpus because they stress short-fragment overlap behavior that modern libraries may not expose.

## Reproducibility

- Machine-readable outputs beside this briefing: `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv`.
- Results are generated from the Lunarc benchmark tree and mirrored into the configured local benchmark archive for later inspection.
