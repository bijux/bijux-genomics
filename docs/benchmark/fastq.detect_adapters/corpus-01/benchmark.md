# `fastq.detect_adapters` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:26:10.835428+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.detect_adapters/lunarc`
- Scenario: `detect_adapters_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `fastqc`
- inspection_mode: `evidence_only`
- evidence_scope: `full_input`
- evidence_format: `fastqc_summary`
- report_only: `True`

## Executive Summary

- Fastest median runtime: `fastqc` at `7.616` seconds.
- Highest mean candidate-adapter count: `fastqc` at `1.000` candidates per sample.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Mean candidates | Mean adapter-trimmed fraction | Median mean Q |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fastqc` | 20 | 100.0% | 7.616 | 1.000 | n/a | 0.000 |

## Cohort Coverage

| Cohort | Samples |
| --- | ---: |
| `ancient_pe` | 5 |
| `ancient_se` | 5 |
| `modern_pe` | 5 |
| `modern_se` | 5 |

## Notes

- This stage is evidence-only: governed benchmarking confirms adapter-inspection throughput and signal stability without mutating reads.
- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection.
