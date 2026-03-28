# `fastq.extract_umis` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:56:48.313237+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.extract_umis/lunarc`
- Scenario: `umi_extraction_fairness`
- Samples benchmarked: `10` paired-end inputs
- Era balance: `5` ancient, `5` modern
- Tools: `umi_tools`
- umi_pattern: `NNNNNNNN`
- allow_missing_umi_headers: `True`

## Executive Summary

- Median runtime: `umi_tools` at `56.993` seconds.
- Mean read retention: `umi_tools` at `1.000`.
- Mean reads with detected UMI: `umi_tools` at `5104575.4` reads.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median read retention | Mean reads with UMI | Mean reads with UMI fraction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `umi_tools` | 10 | 100.0% | 56.993 | 1.000 | 5104575.4 | 1.000 |

## Notes

- This paired-only benchmark keeps the governed UMI parsing contract explicit, so later pattern changes cannot silently invalidate comparisons.
- `corpus-01` is not a native UMI cohort, so the dossier records whether missing-header bypass was enabled during execution.
- Published per-sample rows keep read-retention and UMI-detection behavior auditable alongside runtime.
