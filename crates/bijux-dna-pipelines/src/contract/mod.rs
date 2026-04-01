//! Shared pipeline profile models.

mod capabilities;
mod invariants;
mod profile;

pub use capabilities::{
    ArtifactType, Domain, MetricsBundle, PipelineCapabilities, PipelineContract, ReportSection,
    StabilityTier,
};
pub use invariants::{
    InvariantSeverity, InvariantViolationV1, InvariantsPreset, InvariantsReportV1,
};
pub use profile::{EffectiveDefaults, PipelineProfile, ProfileManifestV1};
