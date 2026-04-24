# Architecture

## Tree
- `src/public_api/` mirrors the curated stable surface and root reexports, with a dedicated stable-surface owner instead of a single-file wrapper.
- `src/cross.rs` and `src/fastq.rs` provide explicit root module entrypoints that re-export their owned directory modules.
- `src/contract/` owns pipeline profile, manifest, capability, effective-default, invariant, and vocabulary contracts, with `stable_surface.rs` owning the aggregated contract exports.
- `src/contract/projections/` separates defaults-ledger, pipeline-contract, and manifest/hash projections.
- `src/defaults/` owns defaults ledgers, typed default-parameter envelopes, empty-param markers, serde codecs, and override merging, with `stable_surface.rs` owning the aggregated defaults exports.
- `src/defaults/merge/` separates merge orchestration from override application and override validation.
- `src/defaults/serde_codec/` separates defaults serialization from defaults deserialization.
- `src/registry/families/` owns domain-partitioned profile family assembly.
- `src/registry/catalog/` owns registry assembly through `pipeline_registry.rs` plus separated profile-by-domain and profile-by-stability query behavior.
- `src/registry/` owns pipeline id validation and lookup entrypoints, with `stable_surface.rs` separating root exports and `profile_lookup/` separating lookup entrypoints from concrete cross/vcf lookup families.
- `src/cross/fastq_to_bam/` owns the cross-domain FASTQ-to-BAM profile family, with `stable_surface.rs` owning the cross-profile exports.
- `src/cross/fastq_to_bam/` separates source-profile loading from merged-default assembly before cross-domain profiles compose their BAM handoff stages.
- `src/cross/fastq_to_bam/profiles/` separates modern and ancient-DNA cross-domain profile families.
- `src/fastq/defaults/` owns stage order, preprocess vs analysis tool maps, preprocess vs analysis params, rationale assembly, and preset override policy, with `defaults_assembly.rs` owning the assembled defaults contract and adna and reference-grade overrides split by tools, params, and rationales.
- `src/fastq/profiles/` owns FASTQ profile identity, profile contracts, profile lookup, and separate default, minimal, adna, and reference-grade aDNA profile families, with `stable_surface.rs` owning the exported FASTQ profile surface and stable ids separated from lookup behavior.
- `src/fastq/invariants/` owns FASTQ validation reports, violation builders, typed stage-parameter access, separated requirement-rule families, and separated adna vs reference-grade preset rules, with `stage_scope.rs` owning shared required-stage scope.
- `src/bam/` and `src/vcf/` own the remaining domain-specific profile definitions and invariants.

## Data flow
1. Domain profile modules define canonical profiles, defaults, and invariants.
2. FASTQ defaults assemble preprocess and analysis contracts separately before merge orchestration validates preset overrides and profile families compose them.
3. `registry/families/` assembles those profiles by domain and `registry/catalog/` turns them into stable queryable collections.
4. `contract` and `defaults` project the profiles into manifests, ledgers, effective defaults, and override behavior for downstream crates.
