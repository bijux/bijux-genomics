use bijux_dna_core::ids::{AssayKind, LibraryLayout, UdgTreatment};
use bijux_dna_core::prelude::id_catalog;

use super::profile_contracts::{fastq_capabilities, fastq_library_model};
use crate::fastq::defaults::{
    adna_fastq_defaults, append_stage_once, default_shotgun_required_stages,
};
use crate::{InvariantsPreset, PipelineId, PipelineProfile, StabilityTier};

#[must_use]
pub fn fastq_adna_profile() -> PipelineProfile {
    let defaults = adna_fastq_defaults();
    let mut required_stages = default_shotgun_required_stages();
    append_stage_once(&mut required_stages, id_catalog::FASTQ_TRIM_TERMINAL_DAMAGE);
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
        capabilities: fastq_capabilities(id_catalog::PIPELINE_FASTQ_ADNA, required_stages),
    }
}
