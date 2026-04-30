# bijux-dna-pipelines Public API

The public API is the stable contract surface exported from `src/lib.rs` and mirrored through `src/public_api/`.

## Public Modules
- `bam`
- `contract`
- `cross`
- `defaults`
- `fastq`
- `public_api`
- `registry`
- `vcf`

## Root Reexports
- Pipeline identity and registry validation: `PipelineId`, `validate_pipeline_id`, `validate_pipeline_id_str`.
- Profile contracts: `PipelineProfile`, `PipelineProfileV1`, `PipelineContract`, `ProfileManifestV1`, `PipelineCapabilities`.
- Defaults contracts: `DefaultParams`, `DefaultProvenanceV1`, `DefaultsLedgerV1`, `EffectiveDefaults`, `EmptyParams`, `merge_effective_defaults`.
- Vocabulary contracts: `ArtifactType`, `Domain`, `InvariantsPreset`, `InvariantSeverity`, `InvariantViolationV1`, `InvariantsReportV1`, `MetricsBundle`, `ReportSection`, `StabilityTier`.
- Shared stage identifiers: `STAGE_CORE_PREPARE_REFERENCE`, `STAGE_CROSS_ALIGN_STUB`.

## Stability Rules
- `src/public_api/stable_surface.rs` must mirror the durable root exports consumed by downstream crates.
- New public modules require a docs update here and an architecture-tree review.
- Public contract field changes require snapshot review and an explicit pipeline contract change.

## Stability Tiers

- Stable: the public modules and root reexports documented in this file.
- Experimental: profile/template helpers are experimental until they are promoted into the documented root reexports or stable module contracts.
- Internal: composition helpers and defaults plumbing that are not re-exported through `src/lib.rs` or `src/public_api/stable_surface.rs`.
