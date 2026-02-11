# bijux-dna-db-ena

Typed ENA downloader crate for retrieving metadata and files by project (`PRJ*`), sample (`SAME*`), and mixed accession lists.

## CLI quick start

Query only (metadata manifest):

```bash
cargo run -p bijux-dna-db-ena -- query \
  --project PRJEB22390 \
  --sample SAMEA7497549 \
  --manifest-out artifacts/ena/manifest.json
```

Download FASTQ with 8 parallel jobs:

```bash
cargo run -p bijux-dna-db-ena -- download \
  --project PRJEB22390 \
  --output-dir artifacts/ena/files \
  --jobs 8 \
  --retries 2
```

## Features

- Multi-input querying: `--project`, `--sample`, and `--accession` can be combined.
- Typed ENA query/result models.
- `read_run` and `analysis` result modes.
- Select file source column: `fastq_ftp`, `submitted_ftp`, `sra_ftp`, `bam_ftp`.
- Protocol preference (`ftp`/`https`) and normalized URLs.
- Parallel downloads with retries.
- Dry-run mode and JSON manifest output.
