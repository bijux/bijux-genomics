# `fastq.deplete_host` on `corpus-01`

## Run Contract

- Generated: 2026-03-28T02:41:03.183632+00:00
- Platform: `lunarc-apptainer`
- Corpus root: `/home/bijan/lu2024-12-24/.cache/corpus_01`
- Run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.deplete_host/lunarc`
- Scenario: `host_depletion_fairness`
- Samples benchmarked: `20`
- Layout balance: `10` single-end, `10` paired-end
- Era balance: `10` ancient, `10` modern
- Tools: `bowtie2`
- reference_index: `/lunarc/nobackup/projects/snic2019-34-3/.cache/extra-data/benchmark/fastq.deplete_host/host_reference/bowtie2_build/index/reference`
- reference_index_digest: `639f1934a7933edae38d8bf42bf9f6cc43560e24686c53b0e94e27f9f6691831`
- reference_index_lineage_json: `/lunarc/nobackup/projects/snic2019-34-3/.cache/extra-data/benchmark/fastq.deplete_host/host_reference/bowtie2_build/index/lineage.json`
- reference_index_lineage_digest: `4680af76346224d129ceba82cb055f78b9936aa8bffd5e405b2b34acb04df872`
- reference_catalog_id: `host_reference`
- reference_index_backend: `bowtie2_build`
- host_identity_threshold: `0.95`
- retain_unmapped_only: `True`

## Executive Summary

- Fastest median runtime: `bowtie2` at `19.256` seconds.
- Highest mean host fraction removed: `bowtie2` at `0.646`.
- Highest median read retention: `bowtie2` at `0.224`.
- Sample failures: `0` sample invocations ended non-zero.

## Tool Summary

| Tool | Samples | Pass rate | Median runtime (s) | Median read retention | Median base retention | Mean host fraction removed | Mean reads removed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `bowtie2` | 20 | 100.0% | 19.256 | 0.224 | 0.246 | 0.646 | 2383476.2 |

## Notes

- `corpus-01` is human DNA, so host depletion here is a deliberately high-pressure false-positive control rather than a permissive cleanup workload.
- The dossier records host index lineage directly in the run manifest so later reruns can detect silent reference drift before comparing removal rates.
