//! Baseline BAM profile definitions.

use bijux_dna_core::ids::{AssayKind, LibraryLayout, LibraryModel, PlatformHint, UdgTreatment};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::defaults::default_params_json;

use super::profile_capabilities::bam_capabilities;
use super::profile_defaults::{defaults_for, stable_bam_stages, to_effective_defaults};
use crate::{Domain, MetricsBundle, PipelineId, PipelineProfile, StabilityTier};

#[must_use]
pub fn bam_default_profile() -> PipelineProfile {
    let stages = stable_bam_stages();
    let defaults = defaults_for(&stages, default_params_json);
    let required_stages: Vec<String> =
        stages.iter().map(|stage| stage.as_str().to_string()).collect();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_BAM_DEFAULT),
        description: "Default BAM pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![Domain::Bam],
        output_domains: vec![Domain::Bam],
        defaults: to_effective_defaults(&defaults),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: LibraryModel {
            layout: LibraryLayout::SingleEnd,
            udg_treatment: UdgTreatment::Unknown,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Unknown,
        },
        capabilities: bam_capabilities(
            id_catalog::PIPELINE_BAM_DEFAULT,
            MetricsBundle::BamCore,
            required_stages,
        ),
    }
}
