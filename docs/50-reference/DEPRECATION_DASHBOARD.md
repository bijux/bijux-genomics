<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-docs -->

# DEPRECATION_DASHBOARD

## Purpose
Generated dashboard of deprecated stage ids, tool ids, metric ids, params, and fields with replacement and migration coverage.

## Scope
Compatibility deprecations declared in `configs/ci/compatibility/deprecations.toml`.

## Non-goals
- Replacing migration playbooks or release planning records.

## Contracts
- Generated-only document; manual edits are forbidden.
- Rows must mirror governed compatibility deprecation declarations.

- Source schema: `bijux.compatibility_deprecations.v1`

| Kind | Subject | Replacement | Deadline | Migration Test Status | Source | Notes |
|---|---|---|---|---|---|---|
| `stage_id` | `fastq.qc_post` | `fastq.report_qc` | `2026-12-31` | `covered by planner and evidence profile contract snapshots` | `governance compatibility inventory` | Legacy qc_post naming remains internal-only; external workflow manifests must use fastq.report_qc. |
| `tool_id` | `bamtools` | `samtools` | `2026-07-01` | `covered by check-deprecations-enforcement` | `configs/ci/registry/deprecations.toml` | Deprecated only for bam.validate. |
| `metric_id` | `runtime_s_legacy` | `runtime_s` | `2026-12-31` | `covered by governed error and schema registry review` | `governance compatibility inventory` | Legacy imported benchmark rows must normalize to runtime_s before publication. |
| `param` | `vcf.impute.legacy_chunk_size` | `chunk_window_size_bp` | `next downstream VCF release` | `covered by check-vcf-deprecation-lifecycle` | `configs/vcf/deprecations/knobs.toml` | Removal phase. |
| `param` | `vcf.phasing.legacy_region` | `chunk_chr_include` | `next downstream VCF release` | `covered by check-vcf-deprecation-lifecycle` | `configs/vcf/deprecations/knobs.toml` | Warn phase. |
| `field` | `artifact_inventory.v0.scientific_context` | `artifact_inventory.v1.scientific_context` | `supported through the current migration window` | `covered by artifact_inventory_reader_accepts_supported_legacy_fixture` | `crates/bijux-dna-runtime/tests/fixtures/runtime_schema/default/artifact_inventory_v0.json` | Legacy artifact inventory payloads omit the explicit scientific_context object. |
