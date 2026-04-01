use bijux_dna_core::ids::LibraryModel;
use serde::Serialize;

use super::{Domain, EffectiveDefaults, InvariantsPreset, PipelineCapabilities, StabilityTier};

#[derive(Debug, Clone, Serialize)]
pub struct PipelineProfile {
    pub id: crate::PipelineId,
    pub description: &'static str,
    pub stability: StabilityTier,
    pub input_domains: Vec<Domain>,
    pub output_domains: Vec<Domain>,
    pub defaults: EffectiveDefaults,
    pub defaults_ledger_ref: &'static str,
    pub invariants_preset: Option<InvariantsPreset>,
    pub library_model: LibraryModel,
    pub capabilities: PipelineCapabilities,
}
