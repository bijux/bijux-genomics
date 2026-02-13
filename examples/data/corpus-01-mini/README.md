# corpus-01-mini

Small iteration corpus scaffold for Lunarc: target `2` SE + `2` PE samples derived from the same ENA selection policy as `corpus-01`.

## Contract
- Raw files are fetched into `raw/` and made read-only.
- Normalized copies are written into `normalized/` as:
  - `sample_0001_R1.fastq.gz`
  - `sample_0001_R2.fastq.gz` (PE only)
- `CHECKSUMS.sha256` stores SHA256 digests for tracked files under `raw/` and `normalized/`.

## Flow
1. `bijux ena select --project PRJEB44430 --target-se 2 --target-pe 2 --out examples/data/corpus-01-mini/ENA_METADATA.snapshot.json`
2. `bijux ena fetch --snapshot examples/data/corpus-01-mini/ENA_METADATA.snapshot.json --out examples/data/corpus-01-mini/raw`
3. `bijux corpus normalize corpus-01-mini`
4. `bijux corpus validate corpus-01-mini`
