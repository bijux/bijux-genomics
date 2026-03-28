# `fastq.trim_terminal_damage` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:45:48.078399+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_terminal_damage/lunarc`
- Scenario: `terminal_damage_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `adapterremoval, cutadapt, seqkit`
- damage_mode: `ancient`
- execution_policy: `explicit_terminal_trim`
- trim_5p_bases: `2`
- trim_3p_bases: `2`

## Executive Summary

- Fastest median runtime: `adapterremoval` at `3.275` seconds.
- Highest median base retention: `cutadapt` at `0.952`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median base retention | Mean asymmetry reduction | Mean Q delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `adapterremoval` | 20 | 100.0% | 3.275 | 0.942 | n/a | 0.000 |
| `cutadapt` | 20 | 100.0% | 7.717 | 0.952 | n/a | 0.000 |
| `seqkit` | 20 | 100.0% | 3.799 | 0.947 | n/a | 0.000 |

## Cohort Coverage

| Cohort | Samples |
| --- | ---: |
| `ancient_pe` | 5 |
| `ancient_se` | 5 |
| `modern_pe` | 5 |
| `modern_se` | 5 |

## Notes

- This benchmark pins the governed ancient-DNA terminal-trim policy across the full corpus so modern samples act as negative-control inputs for damage-aware trimming behavior.
- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection.
