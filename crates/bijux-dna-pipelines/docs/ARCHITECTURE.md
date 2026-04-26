# Architecture

`bijux-dna-pipelines` owns canonical pipeline profiles, defaults, manifests, and registry lookup contracts. It does not execute pipelines.

## Source Layout
- `src/lib.rs` exposes the stable module tree and root reexports.
- `src/public_api/` mirrors the curated stable surface for downstream imports.
- `src/contract/` owns profile, manifest, capability, invariant, projection, and vocabulary contracts.
- `src/defaults/` owns defaults ledgers, typed parameter envelopes, serialization, and override merging.
- `src/registry/` owns pipeline ID validation, family assembly, sorted registry catalogs, and lookup entrypoints.
- `src/fastq/`, `src/bam/`, and `src/vcf/` own domain-specific profile definitions and invariants.
- `src/cross/fastq_to_bam/` owns cross-domain FASTQ-to-BAM profile composition.

## Internal Partitions
- `src/defaults/merge/` separates merge orchestration, override application, and validation.
- `src/defaults/serde_codec/` separates serialization from deserialization.
- `src/contract/projections/` separates defaults-ledger, manifest, and pipeline-contract projections.
- `src/registry/families/` assembles profiles by domain family.
- `src/registry/catalog/` turns assembled profiles into sorted queryable collections through `pipeline_registry.rs`, `profiles_by_domain.rs`, and `profiles_by_stability.rs`.
- `src/registry/profile_lookup/` separates lookup dispatch from concrete cross and VCF lookup families.
- `src/fastq/defaults/` separates stage order, tools, parameters, rationale assembly, and preset overrides.
- `src/fastq/profiles/` separates baseline, minimal, ancient-DNA, and reference-grade FASTQ profiles.
- `src/fastq/invariants/` separates report contracts, typed stage-parameter access, required-stage rules, and preset-specific rules.

## Data Flow
1. Domain modules build canonical profile structs with typed IDs, stages, defaults, capabilities, and invariants.
2. Defaults modules assemble parameter envelopes and provenance, then merge explicit overrides with validation.
3. Registry families collect domain and cross-domain profiles.
4. Registry catalogs sort profiles deterministically and expose list, domain, stability, and lookup queries.
5. Contract projections derive manifests, ledgers, hashes, and downstream-facing contract views.

## Layout Guard
`tests/boundaries/architecture_tree.rs` locks the source and test tree. Any tree change should update this document and the guard in the same intent.
