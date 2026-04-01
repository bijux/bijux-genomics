# Architecture

## Tree
- `src/public_api/` mirrors the curated stable surface and root reexports, with a dedicated stable-surface owner instead of a single-file wrapper.
- `src/contract/` owns pipeline profile, capability, invariant, and projection contracts.
- `src/defaults/` owns defaults ledgers, parameter envelopes, serde codecs, and override merging.
- `src/registry/families/` owns domain-partitioned profile family assembly.
- `src/registry/catalog/` owns registry assembly plus separated profile-by-domain and profile-by-stability query behavior.
- `src/registry/` owns pipeline id validation and id/domain lookup entrypoints over those registry namespaces.
- `src/cross/fastq_to_bam/` owns the cross-domain FASTQ-to-BAM profile family.
- `src/cross/fastq_to_bam/profiles/` separates modern and ancient-DNA cross-domain profile families.
- `src/fastq/defaults/` owns stage order, preprocess vs analysis tool maps, preprocess vs analysis params, rationale assembly, and preset override policy.
- `src/fastq/profiles/` owns FASTQ profile identity, profile contracts, profile lookup, and separate default, minimal, adna, and reference-grade aDNA profile families.
- `src/fastq/invariants/` owns FASTQ validation reports, stage-requirement policy, typed stage-parameter access, and separated adna vs reference-grade preset rules.
- `src/bam/` and `src/vcf/` own the remaining domain-specific profile definitions and invariants.

## Data flow
1. Domain profile modules define canonical profiles, defaults, and invariants.
2. FASTQ defaults assemble preprocess and analysis contracts separately before profile families compose them.
3. `registry/families/` assembles those profiles by domain and `registry/catalog/` turns them into stable queryable collections.
4. `contract` and `defaults` project the profiles into manifests, ledgers, and override behavior for downstream crates.
