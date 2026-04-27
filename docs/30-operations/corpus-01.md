# corpus-01

`corpus-01` is the Lunarc benchmark FASTQ corpus for human DNA stage benchmarking.

## Purpose
Define the stable benchmark corpus used by governed FASTQ performance and integrity checks.

## Scope
This document covers corpus composition, materialization, and the committed corpus contract.

## Non-goals
- Replacing the machine-readable corpus specification.
- Describing ad hoc local corpora outside the governed `corpus-01` contract.

## Contracts
- Corpus membership must stay aligned with
  [configs/runtime/corpora/corpus-01.toml](../../configs/runtime/corpora/corpus-01.toml).
- Materialized files must remain under the configured corpus root, not under source directories.

It is intentionally curated instead of randomly sampled so the benchmark surface stays stable over time:

- 20 total samples
- 10 ancient DNA samples and 10 modern DNA samples
- 5 single-end and 5 paired-end samples in each cohort
- all samples are `Homo sapiens`
- size variation spans compact, mid-size, and larger sub-gigabyte inputs

The selection contract lives in
[configs/runtime/corpora/corpus-01.toml](../../configs/runtime/corpora/corpus-01.toml).

## Composition

- Ancient single-end: `ERR769610`, `ERR769594`, `ERR769585`, `ERR769590`, `ERR769591`
- Ancient paired-end: `ERR4210542`, `ERR4210492`, `ERR15108349`, `ERR15886307`, `ERR15886310`
- Modern single-end: `DRR001066`, `DRR001059`, `DRR001076`, `DRR001083`, `DRR001073`
- Modern paired-end: `DRR015568`, `DRR015482`, `DRR000093`, `DRR000095`, `DRR000550`

Declared size bands:

- `under_100mb`: 12 samples
- `under_500mb`: 7 samples
- `under_1000mb`: 1 sample

## Materialization

The corpus is materialized from ENA metadata and FASTQ URLs through the Bijux CLI:

```bash
cargo run -q -p bijux-dna -- corpus materialize --spec configs/runtime/corpora/corpus-01.toml
```

The materialization root comes from `--root` or from `preferred_root` in
[configs/runtime/corpora/corpus-01.toml](../../configs/runtime/corpora/corpus-01.toml).
The committed spec keeps that path machine-neutral through `${BIJUX_CORPUS_01_ROOT}`, so a new machine only needs to set that environment variable or pass `--root` explicitly.

Materialization writes:

- `CORPUS_SPEC.toml`
- `ENA_METADATA.snapshot.json`
- `MANIFEST.json`
- `raw/`
- `normalized/`

The command validates the curated selection, downloads the raw FASTQs, normalizes the layout, and verifies the final manifest before reporting success.
