//! Shared pipeline profile models.

mod capabilities;
mod effective_defaults;
mod invariants;
mod profile;
mod profile_manifest;
mod projections;

pub use capabilities::{
    ArtifactType, Domain, MetricsBundle, PipelineCapabilities, PipelineContract, ReportSection,
    StabilityTier,
};
pub use effective_defaults::EffectiveDefaults;
pub use invariants::{
    InvariantSeverity, InvariantViolationV1, InvariantsPreset, InvariantsReportV1,
};
pub use profile::PipelineProfile;
pub use profile_manifest::ProfileManifestV1;
