use bijux_dna_core::ids::{AssayKind, LibraryLayout, UdgTreatment};
use bijux_dna_core::prelude::id_catalog;

use super::profile_contracts::{fastq_capabilities, fastq_library_model};
use crate::fastq::defaults::{generic_fastq_defaults, minimal_shotgun_required_stages};
use crate::{PipelineId, PipelineProfile, StabilityTier};

#[must_use]
pub fn fastq_minimal_profile() -> PipelineProfile {
    let required_stages = minimal_shotgun_required_stages();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_MINIMAL),
        description: "Minimal FASTQ pipeline",
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
        capabilities: fastq_capabilities(id_catalog::PIPELINE_FASTQ_MINIMAL, required_stages),
    }
}
