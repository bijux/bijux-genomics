//! Shared pipeline profile models.

mod capabilities;
mod effective_defaults;
mod invariants;
mod profile;
mod projections;

pub use capabilities::{
    ArtifactType, Domain, MetricsBundle, PipelineCapabilities, PipelineContract, ReportSection,
    StabilityTier,
};
pub use invariants::{
    InvariantSeverity, InvariantViolationV1, InvariantsPreset, InvariantsReportV1,
};
pub use effective_defaults::EffectiveDefaults;
pub use profile::{PipelineProfile, ProfileManifestV1};
