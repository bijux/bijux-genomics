//! Shared pipeline profile models.

mod effective_defaults;
mod invariants;
mod pipeline_capabilities;
mod profile;
mod profile_manifest;
mod projections;
mod vocabulary;

pub use effective_defaults::EffectiveDefaults;
pub use invariants::{
    InvariantSeverity, InvariantViolationV1, InvariantsPreset, InvariantsReportV1,
};
pub use pipeline_capabilities::{PipelineCapabilities, PipelineContract};
pub use profile::PipelineProfile;
pub use profile_manifest::ProfileManifestV1;
pub use vocabulary::{ArtifactType, Domain, MetricsBundle, ReportSection, StabilityTier};
