use bijux_dna_core::ids::{AssayKind, LibraryLayout, UdgTreatment};
use bijux_dna_core::prelude::id_catalog;

use super::profile_contracts::{fastq_capabilities, fastq_library_model};
use crate::fastq::defaults::{default_shotgun_required_stages, generic_fastq_defaults};
use crate::{PipelineId, PipelineProfile, StabilityTier};

#[must_use]
pub fn fastq_default_profile() -> PipelineProfile {
    let required_stages = default_shotgun_required_stages();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_DEFAULT),
        description: "Default FASTQ pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults: generic_fastq_defaults(),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: fastq_library_model(
            LibraryLayout::SingleEnd,
            UdgTreatment::Unknown,
            AssayKind::Unknown,
        ),
        capabilities: fastq_capabilities(id_catalog::PIPELINE_FASTQ_DEFAULT, required_stages),
    }
}
