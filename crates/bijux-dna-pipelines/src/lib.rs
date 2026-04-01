//! Pipeline profiles across FASTQ, BAM, and cross-domain workflows.

//! Canonical pipeline profiles and defaults ledger for all domains.

pub mod bam;
pub mod contract;
pub mod cross;
pub mod defaults;
pub mod fastq;
/// Curated mirror of the stable public surface.
pub mod public_api;
pub mod registry;
pub mod vcf;

pub const STAGE_CORE_PREPARE_REFERENCE: &str = "core.prepare_reference";
pub const STAGE_CROSS_ALIGN_STUB: &str = "cross.align_stub";

pub use contract::{
    ArtifactType, Domain, EffectiveDefaults, InvariantSeverity, InvariantViolationV1,
    InvariantsPreset, InvariantsReportV1, MetricsBundle, PipelineCapabilities, PipelineContract,
    PipelineProfile, ProfileManifestV1, ReportSection, StabilityTier,
};
pub use defaults::merge_effective_defaults;
pub use defaults::{DefaultParams, DefaultProvenanceV1, DefaultsLedgerV1, EmptyParams};
pub use registry::{validate_pipeline_id, validate_pipeline_id_str, PipelineId};

pub type PipelineProfileV1 = PipelineProfile;
