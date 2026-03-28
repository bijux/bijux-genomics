# `fastq.trim_polyg_tails` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:39:28.524365+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_polyg_tails/lunarc`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `bbduk, fastp`
- PolyX preset: `illumina_twocolor`
- min_polyg_run: `10`

## Executive Summary

- Fastest median runtime: `bbduk` at `3.191` seconds.
- Highest mean polyG trimming: `fastp` with `34937548.5` bases removed on average.
- Highest median base retention: `bbduk` at `1.000`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median base retention | Mean bases trimmed | Mean Q delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `bbduk` | 20 | 100.0% | 3.191 | 1.000 | 101119.8 | 0.000 |
| `fastp` | 20 | 100.0% | 5.536 | 0.920 | 34937548.5 | 0.000 |

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
