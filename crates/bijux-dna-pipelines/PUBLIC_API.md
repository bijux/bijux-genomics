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
  `defaults/` separates preprocess vs analysis defaults and rationale assembly.
  `profiles/` separates baseline vs ancient-dna profile families.
  `invariants/` separates stage-param access, required rules, and preset rule families.
