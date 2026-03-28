# `fastq.screen_taxonomy` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:36:58.623724+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/results/corpus_01/fastq.screen_taxonomy/lunarc`
- Scenario: `screen_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `centrifuge, kaiju, kraken2, krakenuniq`
- database_root: `/lunarc/nobackup/projects/snic2019-34-3/.cache/extra-data/benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db`
- database_digest: `5bb5be0ced539ee0d1cd88d41f4f63781d8941d1964ef7410c1b16995795f7de`
- database_lineage_json: `/lunarc/nobackup/projects/snic2019-34-3/.cache/extra-data/benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db/lineage.json`
- database_lineage_digest: `6c26e735250bd7438dfe1c2404e4dca6507c98aa0520a2bfd9024b4bebfd92a8`
- database_catalog_id: `taxonomy_reference`
- database_artifact_id: `taxonomy_db`
- database_namespace: `read_screening`
- database_scope: `read_screening`

## Executive Summary

- Fastest median runtime: `kraken2` at `1.392` seconds.
- Lowest mean contamination rate: `centrifuge` at `0.000`.
- Highest mean classified fraction: `centrifuge` at `0.000`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Mean contamination rate | Mean classified fraction | Mean unclassified fraction | Most common top taxon |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `centrifuge` | 20 | 100.0% | 4.016 | 0.000 | 0.000 | 0.000 | `UniVec` |
| `kaiju` | 20 | 100.0% | 2.874 | 0.000 | 0.000 | 0.000 | `unclassified` |
| `kraken2` | 20 | 100.0% | 1.392 | 0.000 | 0.000 | 1.000 | `unclassified` |
| `krakenuniq` | 20 | 100.0% | 2.622 | 0.000 | 0.000 | 1.000 | `unclassified` |

## Notes

- `corpus-01` is human DNA, so the classifier outputs here are mainly a governed background-screening control, not a discovery cohort.
- The dossier records taxonomy database lineage directly in the run manifest so classifier comparisons remain interpretable when the database changes over time.
