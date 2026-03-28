# `fastq.deplete_rrna` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:33:37.303465+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.deplete_rrna/lunarc`
- Scenario: `rrna_depletion_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `sortmerna`
- rrna_bundle_id: `sortmerna_v4_3_default_db`
- rrna_bundle_digest: `sha256:50e80a1a2d1e8c4e2265d3d84c082f258166d4d8f628cd0956c04b15c5fc1a53`
- rrna_db: `/home/bijan/bijux/reference/rrna/sortmerna_v4_3_default_db.fasta`
- min_identity: `0.95`

## Executive Summary

- Fastest median runtime: `sortmerna` at `84.631` seconds.
- Highest mean rRNA fraction removed: `sortmerna` at `0.002`.
- Highest median read retention: `sortmerna` at `1.000`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median read retention | Median base retention | Mean rRNA fraction removed | Mean reads removed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `sortmerna` | 20 | 100.0% | 84.631 | 1.000 | 1.000 | 0.002 | 9646.2 |

## Notes

- `corpus-01` is a human DNA cohort, so rRNA depletion behaves primarily as a governed false-positive control and throughput benchmark rather than a high-yield cleanup stage.
- The run manifest pins the concrete SortMeRNA reference path and bundle digest so future reruns can detect reference drift immediately.
