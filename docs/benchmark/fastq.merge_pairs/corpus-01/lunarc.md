# `fastq.merge_pairs` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:27:36.667006+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.merge_pairs/lunarc`
- Scenario: `merge_fairness`
- Samples benchmarked: `10` paired-end inputs
- Era balance: `5` ancient, `5` modern
- Tool roster: `adapterremoval, bbmerge, flash2, leehom, pear, vsearch`
- merge_overlap: `governed tool default`
- min_length: `governed tool default`
- unmerged_read_policy: `emit_unmerged_pairs`

## Executive Summary

- Fastest median runtime: `vsearch` at `10.166` seconds.
- Highest median merge rate: `adapterremoval` at `0.873`.
- Highest median base retention: `adapterremoval` at `0.325`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median merge rate | Median base retention | Mean merged reads |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `adapterremoval` | 10 | 100.0% | 19.119 | 0.873 | 0.325 | 1040046.9 |
| `bbmerge` | 10 | 100.0% | 55.430 | 0.567 | 0.263 | 425296.9 |
| `flash2` | 10 | 100.0% | 12.107 | 0.531 | 0.209 | 715652.2 |
| `leehom` | 10 | 100.0% | 85.728 | 0.402 | 0.143 | 637122.4 |
| `pear` | 10 | 100.0% | 72.144 | 0.211 | 0.123 | 245063.3 |
| `vsearch` | 10 | 100.0% | 10.166 | 0.121 | 0.070 | 187280.8 |

## Cohort Coverage

| Cohort | Samples |
| --- | ---: |
| `ancient_pe` | 5 |
| `modern_pe` | 5 |

## Notes

- This dossier is intentionally paired-end only. Single-end corpus members are excluded because `fastq.merge_pairs` is not defined for them.
- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection.
