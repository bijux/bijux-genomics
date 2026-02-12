# ARCHITECTURE

## Modules
- `client`: ENA query URL and filereport parsing.
- `model`: domain normalization helpers.
- `download`: task generation + downloader.

## Contract
- Input selection decisions are explicit and serializable.
- Download layout is deterministic from accession + root.
