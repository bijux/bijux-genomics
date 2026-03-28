# `fastq.deplete_reference_contaminants` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:42:07.063262+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.deplete_reference_contaminants/lunarc`
- Scenario: `contaminant_depletion_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `bowtie2`
- reference_index: `/lunarc/nobackup/projects/snic2019-34-3/.cache/bijux-reference/contaminants/phix_and_spikeins/bowtie2/reference`
- reference_index_digest: `8f0f646e6e37ed01b2a33291e96c2a38eeedeb3ca6f8175b685bd6f98719fc8f`
- reference_catalog_id: `contaminant_reference`
- reference_index_backend: `bowtie2_build`
- decoy_mode: `phix_and_spikeins`

## Executive Summary

- Fastest median runtime: `bowtie2` at `18.534` seconds.
- Highest mean contaminant fraction removed: `bowtie2` at `0.000`.
- Highest median read retention: `bowtie2` at `1.000`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median read retention | Median base retention | Mean contaminant fraction removed | Mean reads removed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `bowtie2` | 20 | 100.0% | 18.534 | 1.000 | 1.000 | 0.000 | 0.2 |

## Notes

- `corpus-01` is a human DNA cohort, so contaminant depletion here functions as a false-positive control and reference-lineage throughput benchmark rather than a high-yield cleanup stage.
- The dossier records index lineage and governed decoy policy directly so later reruns can separate reference drift from real backend differences.
