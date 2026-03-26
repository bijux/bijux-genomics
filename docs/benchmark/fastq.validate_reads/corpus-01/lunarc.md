# `fastq.validate_reads` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.validate_reads` stage across all supported backends on the curated `corpus-01` human DNA benchmark set.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `/home/bijan/bijux/corpus_01`
- Benchmark root: `/home/bijan/bijux/corpus_01/benchmarks/fastq.validate_reads/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `fastqvalidator, fastqc, fastq_scan, seqtk, fqtools`
- Execution profile: one benchmark sample at a time, one worker, one thread per tool invocation

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `fastq_scan` is the fastest backend at `p50=0.321s`, while `fastqc` is the slowest at `p50=2.858s`.
- The median slowdown of `fastqc` relative to the fastest backend is `x8.89`.
- Runtime is dominated by modern paired-end samples: for `fastqc`, `modern_pe` averages `32.366s` while `modern_se` averages `0.534s`.
- Input size is the primary cost driver: `fastqc` averages `30.698s` on `under_500mb` samples versus `3.566s` on `under_100mb` samples.
- Correctness stayed stable across all `100` tool-sample observations: `exit_code=0` on `100` rows and `reads_invalid=0` on `100` rows.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median slowdown |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fastq_scan` | 100.0% | 3.803 | 0.321 | 15.765 | 17.006 | x1.00 |
| `fastqvalidator` | 100.0% | 3.877 | 0.329 | 15.924 | 18.162 | x1.02 |
| `fqtools` | 100.0% | 3.839 | 0.332 | 15.964 | 17.180 | x1.03 |
| `seqtk` | 100.0% | 3.816 | 0.338 | 15.718 | 17.078 | x1.05 |
| `fastqc` | 100.0% | 12.911 | 2.858 | 48.258 | 51.606 | x8.89 |

## Cohort behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Samples |
| --- | --- | ---: | ---: | ---: |
| `fastq_scan` | `ancient_pe` | 4.893 | 1.885 | 5 |
| `fastq_scan` | `ancient_se` | 0.149 | 0.150 | 5 |
| `fastq_scan` | `modern_pe` | 10.027 | 15.765 | 5 |
| `fastq_scan` | `modern_se` | 0.145 | 0.146 | 5 |
| `fastqc` | `ancient_pe` | 18.205 | 9.488 | 5 |
| `fastqc` | `ancient_se` | 0.540 | 0.537 | 5 |
| `fastqc` | `modern_pe` | 32.366 | 48.258 | 5 |
| `fastqc` | `modern_se` | 0.534 | 0.542 | 5 |
| `fastqvalidator` | `ancient_pe` | 4.927 | 1.857 | 5 |
| `fastqvalidator` | `ancient_se` | 0.148 | 0.146 | 5 |
| `fastqvalidator` | `modern_pe` | 10.285 | 15.924 | 5 |
| `fastqvalidator` | `modern_se` | 0.149 | 0.152 | 5 |
| `fqtools` | `ancient_pe` | 4.971 | 1.876 | 5 |
| `fqtools` | `ancient_se` | 0.148 | 0.145 | 5 |
| `fqtools` | `modern_pe` | 10.094 | 15.964 | 5 |
| `fqtools` | `modern_se` | 0.142 | 0.139 | 5 |
| `seqtk` | `ancient_pe` | 4.932 | 1.870 | 5 |
| `seqtk` | `ancient_se` | 0.146 | 0.143 | 5 |
| `seqtk` | `modern_pe` | 10.041 | 15.718 | 5 |
| `seqtk` | `modern_se` | 0.146 | 0.145 | 5 |

## Size-band behavior

| Tool | Size band | Mean runtime (s) | Median runtime (s) | Samples |
| --- | --- | ---: | ---: | ---: |
| `fastq_scan` | `under_1000mb` | 0.132 | 0.132 | 1 |
| `fastq_scan` | `under_100mb` | 0.576 | 0.153 | 12 |
| `fastq_scan` | `under_500mb` | 9.861 | 11.152 | 7 |
| `fastqc` | `under_1000mb` | 0.542 | 0.542 | 1 |
| `fastqc` | `under_100mb` | 3.566 | 0.549 | 12 |
| `fastqc` | `under_500mb` | 30.698 | 35.022 | 7 |
| `fastqvalidator` | `under_1000mb` | 0.137 | 0.137 | 1 |
| `fastqvalidator` | `under_100mb` | 0.576 | 0.160 | 12 |
| `fastqvalidator` | `under_500mb` | 10.071 | 11.248 | 7 |
| `fqtools` | `under_1000mb` | 0.139 | 0.139 | 1 |
| `fqtools` | `under_100mb` | 0.580 | 0.159 | 12 |
| `fqtools` | `under_500mb` | 9.954 | 11.413 | 7 |
| `seqtk` | `under_1000mb` | 0.139 | 0.139 | 1 |
| `seqtk` | `under_100mb` | 0.579 | 0.159 | 12 |
| `seqtk` | `under_500mb` | 9.891 | 11.278 | 7 |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) |
| --- | --- | --- | --- | --- | ---: | --- | ---: |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 121.031 | `fastqc` | 51.606 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 113.339 | `fastqc` | 49.398 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 112.043 | `fastqc` | 48.258 |
| `sample_0013` | `ERR15886310` | `ancient` | `pe` | `under_500mb` | 80.113 | `fastqc` | 35.022 |
| `sample_0012` | `ERR15886307` | `ancient` | `pe` | `under_500mb` | 64.554 | `fastqc` | 29.514 |

## Interpretation

- The four stream-oriented validators cluster tightly in median runtime, so they are operationally interchangeable on latency for this corpus.
- `fastqc` remains useful as a richer structural probe, but it carries a clear throughput penalty and should not be treated as a low-latency default validator.
- The benchmark is stable on correctness for this corpus because every backend reported `reads_invalid=0` and `exit_code=0` across all `100` observations.

## Reproducibility

- Primary machine-readable artifacts beside this report: `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`.
- Input cohort metadata is joined through the committed `corpus-01` spec and the materialized corpus manifest, so accession-to-sample identity is stable across rerenders.
