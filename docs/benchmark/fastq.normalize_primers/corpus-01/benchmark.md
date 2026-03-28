# `fastq.normalize_primers` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:32:33.900712+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.normalize_primers/lunarc`
- Scenario: `primer_normalization_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tool roster: `cutadapt`
- primer_set_id: `16S_universal_v1`
- orientation_policy: `normalize_to_forward_primer`
- max_mismatch_rate: `0.1`
- min_overlap_bp: `10`
- strict_5p_anchor: `True`
- allow_iupac_codes: `True`

## Executive Summary

- Fastest median runtime: `cutadapt` at `9.972` seconds.
- Highest mean primer-trimmed fraction: `cutadapt` at `0.000`.
- Highest median forward-orientation fraction: `cutadapt` at `0.000`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median read retention | Mean primer-trimmed fraction | Median forward-orientation fraction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `cutadapt` | 20 | 100.0% | 9.972 | 1.000 | 0.000 | 0.000 |

## Notes

- `corpus-01` is a human DNA cohort, so this run functions as both a throughput benchmark and a governed false-positive control for primer-aware normalization.
- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection.
