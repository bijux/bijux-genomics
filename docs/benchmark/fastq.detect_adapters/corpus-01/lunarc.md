# `fastq.detect_adapters` benchmark on `corpus-01`

## What was run

This benchmark measures the governed `fastq.detect_adapters` stage across the curated `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `/home/bijan/bijux/corpus_01`
- Benchmark root: `/home/bijan/bijux/corpus_01/benchmarks/fastq.detect_adapters/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `fastqc`
- Evidence contract: `evidence_only`, `full_input`, `fastqc_summary`, report_only=`True`
- Execution profile: one benchmark sample at a time, one worker, one thread per tool invocation

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `fastqc` ran at `p50=2.603s` with mean candidate-adapter count `1.000`.
- Runtime remains input-driven for `fastqc`: `modern_pe` averages `20.989s` while `ancient_se` averages `0.518s`.
- Size-band spread stays visible in the observer stage for `fastqc`: `under_500mb` averages `19.859s` versus `2.810s` on `under_100mb` inputs.
- Correctness stayed stable across all `20` tool-sample observations: `exit_code=0` on `20` rows, and the stage preserved `reads_out == reads_in` and `bases_out == bases_in` for every published row.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Mean candidates | Mean trimmed fraction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `fastqc` | 100.0% | 8.663 | 2.603 | 31.576 | 32.589 | 1.000 | n/a |

## Cohort behavior

| Tool | Dimension | Cohort | Samples | Mean runtime (s) | Median runtime (s) | Mean candidates | Mean trimmed fraction |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `fastqc` | `era_layout` | `ancient_pe` | 5 | 12.600 | 7.624 | 1.000 | n/a |
| `fastqc` | `era_layout` | `ancient_se` | 5 | 0.518 | 0.526 | 1.000 | n/a |
| `fastqc` | `era_layout` | `modern_pe` | 5 | 20.989 | 31.576 | 1.000 | n/a |
| `fastqc` | `era_layout` | `modern_se` | 5 | 0.546 | 0.526 | 1.000 | n/a |
| `fastqc` | `size_band` | `under_1000mb` | 1 | 0.538 | 0.538 | 1.000 | n/a |
| `fastqc` | `size_band` | `under_100mb` | 12 | 2.810 | 0.584 | 1.000 | n/a |
| `fastqc` | `size_band` | `under_500mb` | 7 | 19.859 | 23.576 | 1.000 | n/a |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Slowest runtime (s) | Candidate count |
| --- | --- | --- | --- | --- | ---: | ---: |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 32.589 | 1.000 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 31.619 | 1.000 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 31.576 | 1.000 |
| `sample_0013` | `ERR15886310` | `ancient` | `pe` | `under_500mb` | 23.576 | 1.000 |
| `sample_0012` | `ERR15886307` | `ancient` | `pe` | `under_500mb` | 18.602 | 1.000 |
| `sample_0011` | `ERR15108349` | `ancient` | `pe` | `under_100mb` | 7.624 | 1.000 |
| `sample_0014` | `ERR4210492` | `ancient` | `pe` | `under_100mb` | 7.600 | 1.000 |
| `sample_0015` | `ERR4210542` | `ancient` | `pe` | `under_100mb` | 5.598 | 1.000 |
| `sample_0010` | `DRR015568` | `modern` | `pe` | `under_100mb` | 4.594 | 1.000 |
| `sample_0009` | `DRR015482` | `modern` | `pe` | `under_100mb` | 4.567 | 1.000 |

## Interpretation

- `fastq.detect_adapters` is an observer stage, so throughput and signal coverage matter more than retention metrics. The governed contract intentionally preserves input reads and bases unchanged.
- Because the current governed benchmark cohort is a single backend, this dossier is primarily a run-to-run stability baseline across corpus composition rather than a backend ranking exercise.
- `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv` are published beside this briefing for reproducible downstream analysis.
