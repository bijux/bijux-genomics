# `fastq.filter_low_complexity` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:43:15.299292+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.filter_low_complexity/lunarc`
- Scenario: `low_complexity_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `bbduk, prinseq`
- entropy_threshold: `0.55`
- polyx_threshold: `unset`

## Executive Summary

- Fastest median runtime: `bbduk` at `3.468` seconds.
- Highest median base retention: `bbduk` at `0.999`.
- Highest mean low-complexity removals: `bbduk` at `245897.5` reads.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median base retention | Median read retention | Mean removed reads | Mean Q delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `bbduk` | 20 | 100.0% | 3.468 | 0.999 | 0.999 | 245897.5 | 0.000 |
| `prinseq` | 20 | 100.0% | 7.380 | 0.999 | 0.999 | 92116.8 | 0.000 |

## Notes

- This corpus benchmark keeps one fixed low-complexity contract across the full roster so any removal differences remain attributable to backend behavior rather than threshold drift.
- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection.
