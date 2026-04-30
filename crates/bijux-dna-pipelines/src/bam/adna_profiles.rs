//! Ancient DNA BAM profile definitions.

use bijux_dna_core::ids::{AssayKind, LibraryLayout, LibraryModel, PlatformHint, UdgTreatment};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::defaults::{adna_capture_params_json, adna_shotgun_params_json};

use super::profile_capabilities::bam_capabilities;
use super::profile_defaults::{
    catalog_bam_stages, defaults_for, filter_downstream, to_effective_defaults,
};
use crate::{Domain, InvariantsPreset, MetricsBundle, PipelineId, PipelineProfile, StabilityTier};

#[must_use]
pub fn bam_adna_shotgun_profile() -> PipelineProfile {
    let mut stages = catalog_bam_stages();
    stages.retain(|stage| *stage != bijux_dna_domain_bam::BamStage::Align);
    stages.retain(|stage| *stage != bijux_dna_domain_bam::BamStage::Recalibration);
    filter_downstream(&mut stages);
    let defaults = defaults_for(&stages, adna_shotgun_params_json);
    let required_stages: Vec<String> =
        stages.iter().map(|stage| stage.as_str().to_string()).collect();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_BAM_ADNA_SHOTGUN),
        description: "Ancient DNA shotgun defaults",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Bam],
        output_domains: vec![Domain::Bam],
        defaults: to_effective_defaults(&defaults),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some(InvariantsPreset::Adna),
        library_model: LibraryModel {
            layout: LibraryLayout::PairedEnd,
            udg_treatment: UdgTreatment::None,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Shotgun,
        },
        capabilities: bam_capabilities(
            id_catalog::PIPELINE_BAM_ADNA_SHOTGUN,
            MetricsBundle::BamAdna,
            required_stages,
        ),
    }
}

#[must_use]
pub fn bam_adna_capture_profile() -> PipelineProfile {
    let mut stages = catalog_bam_stages();
    stages.retain(|stage| *stage != bijux_dna_domain_bam::BamStage::Align);
    stages.retain(|stage| *stage != bijux_dna_domain_bam::BamStage::Recalibration);
    filter_downstream(&mut stages);
    let defaults = defaults_for(&stages, adna_capture_params_json);
    let required_stages: Vec<String> =
        stages.iter().map(|stage| stage.as_str().to_string()).collect();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_BAM_ADNA_CAPTURE),
        description: "Ancient DNA capture defaults",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Bam],
        output_domains: vec![Domain::Bam],
        defaults: to_effective_defaults(&defaults),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some(InvariantsPreset::Adna),
        library_model: LibraryModel {
            layout: LibraryLayout::PairedEnd,
            udg_treatment: UdgTreatment::None,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Capture,
        },
        capabilities: bam_capabilities(
            id_catalog::PIPELINE_BAM_ADNA_CAPTURE,
            MetricsBundle::BamAdna,
            required_stages,
        ),
    }
}

#[must_use]
pub fn bam_adna_profile() -> PipelineProfile {
    bam_adna_shotgun_profile()
}

#[must_use]
pub fn bam_reference_adna_profile() -> PipelineProfile {
    let mut profile = bam_adna_shotgun_profile();
    profile.id = PipelineId::from_static(id_catalog::PIPELINE_BAM_REFERENCE_ADNA);
    profile.description = "Reference-grade ancient DNA BAM defaults";
    profile.capabilities = bam_capabilities(
        id_catalog::PIPELINE_BAM_REFERENCE_ADNA,
        MetricsBundle::BamAdna,
        profile.capabilities.required_stages.clone(),
    );
    profile
}
