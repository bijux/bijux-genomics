<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-compatibility-matrix -->

# COMPATIBILITY_MATRIX

## Purpose
Generated compatibility matrix derived from pipeline profile IDs and tool registry inventory.

## Scope
Profiles sourced from `crates/bijux-dna-core/src/id_catalog/pipeline/`; registries include 87 tool entries.

## Non-goals
- Replacing detailed per-domain migration guides.

## Contracts
- Matrix is generated-only and must not be manually edited.
- Breaking contract changes require version/schema updates and matrix regeneration.

| Pipeline Profile | Domain | Stability | Plan Contract | Report Contract | Compatibility Rule |
|---|---|---|---|---|---|
| `bam-to-bam__adna_capture__v1` | `bam` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `bam-to-bam__adna_shotgun__v1` | `bam` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `bam-to-bam__default__v1` | `bam` | `stable` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `bam-to-bam__reference_adna__v1` | `bam` | `stable` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `bam-to-vcf__default__v1` | `bam` | `stable` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-bam__adna_shotgun__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-bam__default__v1` | `fastq` | `stable` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__adna__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__amplicon_standard__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__amplicon_umi__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__contaminant_depletion__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__default__v1` | `fastq` | `stable` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__edna_metabarcoding__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__host_depletion__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__minimal__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__qc_only__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__reference_adna__v1` | `fastq` | `stable` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__rrna_depletion__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__trim_qc__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-fastq__umi__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `fastq-to-vcf__minimal__v1` | `fastq` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `vcf-to-vcf__minimal__v1` | `vcf` | `experimental` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
| `vcf-to-vcf__reference_basic__v1` | `vcf` | `stable` | `v1` | `v1` | compatible if stage/tool contracts unchanged |
