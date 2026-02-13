# corpus-01

## Scope
- Dataset source: ENA project `PRJEB44430`.
- Composition target: `10` single-end (SE) runs + `10` paired-end (PE) runs.
- Fetch path contract: raw files are written under `corpus-01/raw/`, normalized copies under `corpus-01/normalized/`.

## Selection Rules
- Keep only records with usable FASTQ URLs and complete metadata:
  - accession IDs
  - read layout
  - library strategy/type
  - instrument model
  - base/read counts
- Reject records with unusual or missing layout, incomplete metadata, or out-of-scope size signals.
- Enforce corpus contracts:
  - SE: exactly one FASTQ
  - PE: exactly two FASTQs and paired read names consistent at header level

## Reproducibility
- `ENA_METADATA.snapshot.json` stores the selected and rejected ENA records.
- `CHECKSUMS.sha256` stores SHA256 digests for tracked files under `raw/` and `normalized/`.
- `bijux corpus validate corpus-01` validates layout, checksums, and read-name sanity.
- `bijux corpus list --json` enumerates normalized inputs deterministically.

## Limitations
- ENA metadata quality can vary; strict filtering may reduce eligible runs.
- Read-name consistency check is a lightweight first-header sanity guard, not full read-by-read pair reconciliation.
- Raw corpus inputs are immutable by policy; transforms must produce new files under `normalized/`.
