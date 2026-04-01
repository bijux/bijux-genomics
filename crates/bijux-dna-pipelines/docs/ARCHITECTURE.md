# Architecture

## Tree
- `src/public_api/` mirrors the curated stable surface and root reexports.
- `src/contract/` owns pipeline profile, capability, invariant, and projection contracts.
- `src/defaults/` owns defaults ledgers, parameter envelopes, serde codecs, and override merging.
- `src/registry/families/` owns domain-partitioned profile family assembly.
- `src/registry/catalog/` owns registry assembly and query behavior.
- `src/registry/` owns pipeline id validation and id/domain lookup entrypoints over those registry namespaces.
- `src/cross/fastq_to_bam/` owns the cross-domain FASTQ-to-BAM profile family.
- `src/fastq/defaults/` owns stage order, tool maps, parameter defaults, and preset override policy.
- `src/fastq/profiles/` owns FASTQ profile identity, contract templates, and profile family declarations.
- `src/fastq/invariants/` owns FASTQ invariant reports, baseline rules, and preset-specific rules.
- `src/bam/` and `src/vcf/` own the remaining domain-specific profile definitions and invariants.

## Data flow
1. Domain profile modules define canonical profiles, defaults, and invariants.
2. `registry/families/` assembles those profiles by domain and `registry/catalog/` turns them into stable queryable collections.
3. `contract` and `defaults` project the profiles into manifests, ledgers, and override behavior for downstream crates.
