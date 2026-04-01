# Architecture

## Tree
- `src/public_api/` mirrors the curated stable surface and root reexports.
- `src/contract/` owns pipeline profile, capability, invariant, and projection contracts.
- `src/defaults/` owns defaults ledgers, parameter envelopes, serde codecs, and override merging.
- `src/registry/` owns pipeline id validation, profile collections, registry assembly, and id/domain lookup.
- `src/cross/fastq_to_bam/` owns the cross-domain FASTQ-to-BAM profile family.
- `src/fastq/`, `src/bam/`, and `src/vcf/` own domain-specific profile definitions and invariants.

## Data flow
1. Domain profile modules define canonical profiles and invariants.
2. `registry` assembles those profiles into stable collections and lookup entrypoints.
3. `contract` and `defaults` project the profiles into manifests, ledgers, and override behavior for downstream crates.
