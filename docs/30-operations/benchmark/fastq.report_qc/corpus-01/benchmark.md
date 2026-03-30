# `fastq.report_qc` benchmark on `corpus-01`

## What was run

This benchmark measures the governed `fastq.report_qc` stage across the curated `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

- Platform: `lunarc-apptainer` on Lunarc
- Corpus root: `<REMOTE_CORPUS_ROOT>`
- Benchmark root: `<LOCAL_CACHE_RESULTS_ROOT>/corpus_01/fastq.report_qc/lunarc`
- Input balance: `10` ancient, `10` modern, `10` single-end, `10` paired-end
- Tool set: `multiqc`
- Aggregation contract: `multiqc`, `governed_qc_artifacts`, report_only=`True`
- Governed contributor stages: `fastq.detect_adapters, fastq.profile_read_lengths, fastq.profile_reads, fastq.validate_reads`

## Executive summary

- Every tool completed successfully on all `20` samples; stage-level sample failures were `0`.
- `multiqc` ran at `p50=2.276s` with median MultiQC sample count `1.500` and median module count `11`.
- Governed evidence stayed stable: median governed QC input count was `6` and every published row preserved `reads_out == reads_in` and `bases_out == bases_in`.
- Runtime remains input-driven for `multiqc`: `modern_pe` averages `2.267s` while `ancient_pe` averages `2.286s`.
- Size-band spread is visible in the aggregation stage: `under_500mb` averages `2.249s` versus `2.305s` on `under_100mb` inputs.

## Tool ranking

| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median modules | Median sample count | Median governed inputs | Median contamination |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `multiqc` | 100.0% | 2.274 | 2.276 | 2.356 | 2.483 | 11 | 1.500 | 6 | 0 |

## Cohort behavior

| Tool | Dimension | Cohort | Samples | Mean runtime (s) | Median runtime (s) | Median modules | Median governed inputs | Median contamination |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `multiqc` | `era_layout` | `ancient_pe` | 5 | 2.286 | 2.304 | 11 | 6 | 0 |
| `multiqc` | `era_layout` | `ancient_se` | 5 | 2.361 | 2.349 | 11 | 6 | 0 |
| `multiqc` | `era_layout` | `modern_pe` | 5 | 2.267 | 2.254 | 11 | 6 | 0 |
| `multiqc` | `era_layout` | `modern_se` | 5 | 2.182 | 2.217 | 11 | 6 | 0 |
| `multiqc` | `size_band` | `under_1000mb` | 1 | 2.078 | 2.078 | 11 | 6 | 0 |
| `multiqc` | `size_band` | `under_100mb` | 12 | 2.305 | 2.311 | 11 | 6 | 0 |
| `multiqc` | `size_band` | `under_500mb` | 7 | 2.249 | 2.253 | 11 | 6 | 0 |

## Highest-cost samples

| Sample | Accession | Era | Layout | Size band | Slowest runtime (s) | Modules | Governed inputs |
| --- | --- | --- | --- | --- | ---: | ---: | ---: |
| `sample_0020` | `ERR769610` | `ancient` | `se` | `under_100mb` | 2.483 | 10 | 6 |
| `sample_0018` | `ERR769591` | `ancient` | `se` | `under_500mb` | 2.369 | 11 | 6 |
| `sample_0009` | `DRR015482` | `modern` | `pe` | `under_100mb` | 2.356 | 12 | 6 |
| `sample_0016` | `ERR769585` | `ancient` | `se` | `under_100mb` | 2.349 | 11 | 6 |
| `sample_0014` | `ERR4210492` | `ancient` | `pe` | `under_100mb` | 2.344 | 12 | 6 |
| `sample_0010` | `DRR015568` | `modern` | `pe` | `under_100mb` | 2.337 | 12 | 6 |
| `sample_0019` | `ERR769594` | `ancient` | `se` | `under_100mb` | 2.316 | 10 | 6 |
| `sample_0015` | `ERR4210542` | `ancient` | `pe` | `under_100mb` | 2.306 | 12 | 6 |
| `sample_0011` | `ERR15108349` | `ancient` | `pe` | `under_100mb` | 2.304 | 11 | 6 |
| `sample_0017` | `ERR769590` | `ancient` | `se` | `under_100mb` | 2.288 | 11 | 6 |

## Interpretation

- `fastq.report_qc` is a report-only aggregation stage, so benchmark value comes from stable governed manifest handling, MultiQC bundle publication, and runtime predictability across corpus composition rather than from read mutation.
- The governed input contract here deliberately joins validation, adapter inspection, read profiling, and read-length evidence so the published aggregation reflects the canonical raw-QC surface instead of a single observer shortcut.
- `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv` are published beside this briefing for reproducible downstream analysis.
