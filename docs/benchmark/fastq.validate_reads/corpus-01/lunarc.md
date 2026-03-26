# `fastq.validate_reads` on `corpus-01`

## Run Contract

- Generated: 2026-03-26T01:43:41.422078+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/bijux/corpus_01`
- Run root: `/home/bijan/bijux/corpus_01/benchmarks/fastq.validate_reads/lunarc`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `fastqvalidator, fastqc, fastq_scan, seqtk, fqtools`

## Executive Summary

- Fastest median runtime: `fastq_scan` at `0.321` seconds.
- Highest pass rate: `fastq_scan` at `100.0%`.
- Most invalid reads reported: `fastq_scan` with `0` reads.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Invalid reads | Strict pass rate |
| --- | ---: | ---: | ---: | ---: | ---: |
| `fastq_scan` | 20 | 100.0% | 0.321 | 0 | n/a |
| `fastqc` | 20 | 100.0% | 2.858 | 0 | n/a |
| `fastqvalidator` | 20 | 100.0% | 0.329 | 0 | n/a |
| `fqtools` | 20 | 100.0% | 0.332 | 0 | n/a |
| `seqtk` | 20 | 100.0% | 0.338 | 0 | n/a |

## Cohort Coverage

| Cohort | Samples |
| --- | ---: |
| `ancient_pe` | 5 |
| `ancient_se` | 5 |
| `modern_pe` | 5 |
| `modern_se` | 5 |

## Notes

- Ancient and modern samples are resolved by matching normalized FASTQ checksums back to raw accession directories and then joining those accessions to `configs/runtime/corpora/corpus-01.toml`.
- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection.
