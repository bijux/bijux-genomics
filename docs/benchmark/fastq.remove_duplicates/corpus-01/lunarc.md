# `fastq.remove_duplicates` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:29:13.766283+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.remove_duplicates/lunarc`
- Scenario: `dedup_fairness`
- Samples benchmarked: `10` paired-end inputs
- Era balance: `5` ancient, `5` modern
- Tool roster: `clumpify, fastuniq`
- dedup_mode: `exact`
- keep_order: `True`

## Executive Summary

- Fastest median runtime: `clumpify` at `21.292` seconds.
- Highest median deduplication rate: `clumpify` at `0.088`.
- Highest mean duplicate removal: `clumpify` at `1194672.1` reads.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median dedup rate | Mean duplicate reads |
| --- | ---: | ---: | ---: | ---: | ---: |
| `clumpify` | 10 | 100.0% | 21.292 | 0.088 | 1194672.1 |
| `fastuniq` | 10 | 100.0% | 32.363 | 0.061 | 1064557.7 |

## Notes

- This paired-only benchmark holds one stable deduplication contract across the full cohort so rate differences remain attributable to backend behavior rather than policy drift.
- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection.
