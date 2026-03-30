# `fastq.validate_reads` benchmark on `corpus-01`

## What was run

This benchmark compares the governed `fastq.validate_reads` stage across all supported backends on the curated `corpus-01` human DNA benchmark set.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `<REMOTE_CORPUS_ROOT>`
- Benchmark root: `<LOCAL_CACHE_RESULTS_ROOT>/corpus_01/fastq.validate_reads/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `fastqvalidator, fastqc, fastq_scan, seqtk, fqtools`
- Execution profile: one benchmark sample at a time, one worker, one thread per tool invocation

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `fastq_scan` is the fastest backend at `p50=2.970s`, while `fastqc` is the slowest at `p50=9.419s`.
- The median slowdown of `fastqc` relative to the fastest backend is `x3.17`.
- Runtime is dominated by modern paired-end samples: for `fastqc`, `modern_pe` averages `33.959s` while `modern_se` averages `19.968s`.
- Input size is the primary cost driver: `fastqc` averages `42.414s` on `under_500mb` samples versus `7.457s` on `under_100mb` samples.
- Correctness stayed stable across all `100` tool-sample observations: `exit_code=0` on `100` rows and `reads_invalid=0` on `100` rows.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median slowdown |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fastq_scan` | 100.0% | 8.952 | 2.970 | 26.410 | 28.417 | x1.00 |
| `seqtk` | 100.0% | 9.145 | 2.978 | 26.948 | 29.432 | x1.00 |
| `fqtools` | 100.0% | 9.981 | 3.174 | 28.667 | 31.063 | x1.07 |
| `fastqvalidator` | 100.0% | 12.476 | 3.923 | 34.755 | 38.042 | x1.32 |
| `fastqc` | 100.0% | 21.955 | 9.419 | 51.865 | 55.634 | x3.17 |

## Cohort behavior

| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Samples |
| --- | --- | ---: | ---: | ---: |
| `fastq_scan` | `ancient_pe` | 7.627 | 3.101 | 5 |
| `fastq_scan` | `ancient_se` | 4.683 | 2.038 | 5 |
| `fastq_scan` | `modern_pe` | 16.673 | 26.410 | 5 |
| `fastq_scan` | `modern_se` | 6.824 | 2.729 | 5 |
| `fastqc` | `ancient_pe` | 18.835 | 9.984 | 5 |
| `fastqc` | `ancient_se` | 15.056 | 7.603 | 5 |
| `fastqc` | `modern_pe` | 33.959 | 50.788 | 5 |
| `fastqc` | `modern_se` | 19.968 | 8.853 | 5 |
| `fastqvalidator` | `ancient_pe` | 10.451 | 4.116 | 5 |
| `fastqvalidator` | `ancient_se` | 7.237 | 2.960 | 5 |
| `fastqvalidator` | `modern_pe` | 22.162 | 34.755 | 5 |
| `fastqvalidator` | `modern_se` | 10.055 | 3.731 | 5 |
| `fqtools` | `ancient_pe` | 8.509 | 3.329 | 5 |
| `fqtools` | `ancient_se` | 5.478 | 2.299 | 5 |
| `fqtools` | `modern_pe` | 18.185 | 28.667 | 5 |
| `fqtools` | `modern_se` | 7.752 | 2.880 | 5 |
| `seqtk` | `ancient_pe` | 7.836 | 3.163 | 5 |
| `seqtk` | `ancient_se` | 4.857 | 2.094 | 5 |
| `seqtk` | `modern_pe` | 17.150 | 26.948 | 5 |
| `seqtk` | `modern_se` | 6.737 | 2.558 | 5 |

## Size-band behavior

| Tool | Size band | Mean runtime (s) | Median runtime (s) | Samples |
| --- | --- | ---: | ---: | ---: |
| `fastq_scan` | `under_1000mb` | 18.710 | 18.710 | 1 |
| `fastq_scan` | `under_100mb` | 1.877 | 1.681 | 12 |
| `fastq_scan` | `under_500mb` | 19.686 | 17.049 | 7 |
| `fastqc` | `under_1000mb` | 52.707 | 52.707 | 1 |
| `fastqc` | `under_100mb` | 7.457 | 7.639 | 12 |
| `fastqc` | `under_500mb` | 42.414 | 46.363 | 7 |
| `fastqvalidator` | `under_1000mb` | 28.551 | 28.551 | 1 |
| `fastqvalidator` | `under_100mb` | 2.522 | 2.212 | 12 |
| `fastqvalidator` | `under_500mb` | 27.243 | 25.781 | 7 |
| `fqtools` | `under_1000mb` | 21.832 | 21.832 | 1 |
| `fqtools` | `under_100mb` | 2.014 | 1.746 | 12 |
| `fqtools` | `under_500mb` | 21.947 | 19.501 | 7 |
| `seqtk` | `under_1000mb` | 18.846 | 18.846 | 1 |
| `seqtk` | `under_100mb` | 1.846 | 1.591 | 12 |
| `seqtk` | `under_500mb` | 20.271 | 17.964 | 7 |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) |
| --- | --- | --- | --- | --- | ---: | --- | ---: |
| `sample_0003` | `DRR000550` | `modern` | `pe` | `under_500mb` | 182.586 | `fastqc` | 55.634 |
| `sample_0001` | `DRR000093` | `modern` | `pe` | `under_500mb` | 171.086 | `fastqc` | 51.865 |
| `sample_0002` | `DRR000095` | `modern` | `pe` | `under_500mb` | 167.569 | `fastqc` | 50.788 |
| `sample_0006` | `DRR001073` | `modern` | `se` | `under_1000mb` | 140.646 | `fastqc` | 52.707 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 124.763 | `fastqc` | 46.363 |

## Interpretation

- The four stream-oriented validators cluster tightly in median runtime, so they are operationally interchangeable on latency for this corpus.
- `fastqc` remains useful as a richer structural probe, but it carries a clear throughput penalty and should not be treated as a low-latency default validator.
- The benchmark is stable on correctness for this corpus because every backend reported `reads_invalid=0` and `exit_code=0` across all `100` observations.

## Reproducibility

- Primary machine-readable artifacts beside this report: `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`.
- Input cohort metadata is joined through the committed `corpus-01` spec and the materialized corpus manifest, so accession-to-sample identity is stable across rerenders.
