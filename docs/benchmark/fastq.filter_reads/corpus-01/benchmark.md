# `fastq.filter_reads` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:27:21.590382+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.filter_reads/lunarc`
- Scenario: `filter_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `bbduk, fastp, prinseq, seqkit`
- max_n: `0`
- max_n_fraction: `None`
- max_n_count: `3`
- low_complexity_threshold: `20.0`
- entropy_threshold: `18.0`
- kmer_ref: `None`
- polyx_policy: `trim`

## Executive Summary

- Fastest median runtime: `seqkit` at `1.239` seconds.
- Highest median base retention: `bbduk` at `1.000`.
- Highest mean reads dropped: `fastp` at `284465.7` reads.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median base retention | Median read retention | Mean reads dropped | Mean low-complexity removals | Mean N removals |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `bbduk` | 20 | 100.0% | 3.445 | 1.000 | 1.000 | 0.0 | 0.0 | 0.0 |
| `fastp` | 20 | 100.0% | 4.150 | 0.920 | 0.952 | 284465.7 | 0.0 | 1265.3 |
| `prinseq` | 20 | 100.0% | 1.962 | 1.000 | 1.000 | 0.0 | 0.0 | 0.0 |
| `seqkit` | 20 | 100.0% | 1.239 | 1.000 | 1.000 | 0.0 | 0.0 | 0.0 |

## Notes

- This benchmark keeps one governed filter contract across the full human DNA cohort so any retention drift stays attributable to backend behavior rather than threshold changes.
- Raw backend report formats remain explicit in `sample_results.csv` so future audits can distinguish native evidence from the governed summary layer.
