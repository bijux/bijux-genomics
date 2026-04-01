//! FASTQ pipeline profile definitions.

mod catalog;
mod contract_templates;

use bijux_dna_core::ids::{AssayKind, LibraryLayout, LibraryModel, PlatformHint, UdgTreatment};
use bijux_dna_core::prelude::id_catalog;

use contract_templates::{fastq_capabilities, fastq_library_model};

use super::defaults::{
    adna_fastq_defaults, append_stage_once, default_shotgun_required_stages, fastq_defaults,
    reference_adna_fastq_defaults,
};
use crate::{InvariantsPreset, PipelineId, PipelineProfile, StabilityTier};

pub use catalog::{fastq_profiles_by_id, FASTQ_PROFILE_IDS};

#[must_use]
pub fn fastq_minimal_profile() -> PipelineProfile {
    let required_stages = default_shotgun_required_stages();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_MINIMAL),
        description: "Minimal FASTQ pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults: fastq_defaults(false),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: fastq_library_model(
            LibraryLayout::SingleEnd,
            UdgTreatment::Unknown,
            AssayKind::Unknown,
        ),
        capabilities: fastq_capabilities(required_stages),
    }
}

#[must_use]
pub fn fastq_default_profile() -> PipelineProfile {
    let required_stages = default_shotgun_required_stages();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_DEFAULT),
        description: "Default FASTQ pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults: fastq_defaults(false),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: fastq_library_model(
            LibraryLayout::SingleEnd,
            UdgTreatment::Unknown,
            AssayKind::Unknown,
        ),
        capabilities: fastq_capabilities(required_stages),
    }
}

#[must_use]
pub fn fastq_adna_profile() -> PipelineProfile {
    let defaults = adna_fastq_defaults();
    let mut required_stages = default_shotgun_required_stages();
    append_stage_once(&mut required_stages, id_catalog::FASTQ_MERGE);
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_ADNA),
        description: "aDNA-oriented FASTQ pipeline defaults",
        stability: StabilityTier::Beta,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some(InvariantsPreset::Adna),
        library_model: fastq_library_model(
            LibraryLayout::PairedEnd,
            UdgTreatment::None,
            AssayKind::Shotgun,
        ),
        capabilities: fastq_capabilities(required_stages),
    }
}

#[must_use]
pub fn fastq_reference_adna_profile() -> PipelineProfile {
    let defaults = reference_adna_fastq_defaults();
    let mut required_stages = default_shotgun_required_stages();
    append_stage_once(&mut required_stages, id_catalog::FASTQ_LOW_COMPLEXITY);
    append_stage_once(&mut required_stages, id_catalog::FASTQ_MERGE);
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_REFERENCE_ADNA),
        description: "Reference-grade aDNA FASTQ pipeline defaults",
        stability: StabilityTier::Beta,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some(InvariantsPreset::ReferenceAdna),
        library_model: fastq_library_model(
            LibraryLayout::PairedEnd,
            UdgTreatment::None,
            AssayKind::Shotgun,
        ),
        capabilities: fastq_capabilities(required_stages),
    }
}
