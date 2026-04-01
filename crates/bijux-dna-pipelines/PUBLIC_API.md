# bijux-dna-pipelines Public API

Public modules exported from src/lib.rs:
- bam
- contract
- cross
- defaults
- fastq
- public_api
- registry
- vcf

Stable surface mirrors:
- `public_api` mirrors the durable root modules and root reexports.
- `contract` and `defaults` own the crate-level pipeline vocabulary and defaults behavior.
- `fastq` now exposes a curated FASTQ contract stack over internal `defaults/`, `profiles/`, and `invariants/` namespaces.
