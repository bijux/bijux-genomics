# `fastq.detect_adapters` benchmark on `corpus-01`

## What was run

This benchmark measures the governed `fastq.detect_adapters` stage across the curated `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `<REMOTE_CORPUS_ROOT>`
- Benchmark root: `<LOCAL_CACHE_RESULTS_ROOT>/corpus_01/fastq.detect_adapters/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `fastqc`
- Evidence contract: `evidence_only`, `full_input`, `fastqc_summary`, report_only=`True`
- Execution profile: one benchmark sample at a time, one worker, one thread per tool invocation

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `fastqc` ran at `p50=7.616s` with mean candidate-adapter count `1.000`.
- Runtime remains input-driven for `fastqc`: `modern_pe` averages `22.152s` while `ancient_se` averages `13.831s`.
- Size-band spread stays visible in the observer stage for `fastqc`: `under_500mb` averages `29.865s` versus `6.282s` on `under_100mb` inputs.
- Correctness stayed stable across all `20` tool-sample observations: `exit_code=0` on `20` rows, and the stage preserved `reads_out == reads_in` and `bases_out == bases_in` for every published row.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Mean candidates | Mean trimmed fraction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `fastqc` | 100.0% | 16.403 | 7.616 | 35.158 | 43.624 | 1.000 | n/a |

## Cohort behavior

| Tool | Dimension | Cohort | Samples | Mean runtime (s) | Median runtime (s) | Mean candidates | Mean trimmed fraction |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `fastqc` | `era_layout` | `ancient_pe` | 5 | 12.624 | 7.641 | 1.000 | n/a |
| `fastqc` | `era_layout` | `ancient_se` | 5 | 13.831 | 6.620 | 1.000 | n/a |
| `fastqc` | `era_layout` | `modern_pe` | 5 | 22.152 | 31.757 | 1.000 | n/a |
| `fastqc` | `era_layout` | `modern_se` | 5 | 17.006 | 7.590 | 1.000 | n/a |
| `fastqc` | `size_band` | `under_1000mb` | 1 | 43.624 | 43.624 | 1.000 | n/a |
| `fastqc` | `size_band` | `under_100mb` | 12 | 6.282 | 6.112 | 1.000 | n/a |
| `fastqc` | `size_band` | `under_500mb` | 7 | 29.865 | 31.757 | 1.000 | n/a |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Slowest runtime (s) | Candidate count |
| --- | --- | --- | --- | --- | ---: | ---: |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 43.624 | 1.000 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 42.628 | 1.000 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 35.158 | 1.000 |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 33.643 | 1.000 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 31.757 | 1.000 |
| `sample_0013` | `ERR15886310` | `ancient` | `pe` | `under_500mb` | 23.574 | 1.000 |
| `sample_0008` | `DRR001083` | `modern` | `se` | `under_500mb` | 22.642 | 1.000 |
| `sample_0012` | `ERR15886307` | `ancient` | `pe` | `under_500mb` | 19.652 | 1.000 |
| `sample_0017` | `ERR769590` | `ancient` | `se` | `under_100mb` | 9.629 | 1.000 |
| `sample_0011` | `ERR15108349` | `ancient` | `pe` | `under_100mb` | 7.641 | 1.000 |

## Interpretation

- `fastq.detect_adapters` is an observer stage, so throughput and signal coverage matter more than retention metrics. The governed contract intentionally preserves input reads and bases unchanged.
- Because the current governed benchmark cohort is a single backend, this dossier is primarily a run-to-run stability baseline across corpus composition rather than a backend ranking exercise.
- `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv` are published beside this briefing for reproducible downstream analysis.
