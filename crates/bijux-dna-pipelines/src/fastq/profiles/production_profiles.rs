use bijux_dna_core::ids::{AssayKind, LibraryLayout, UdgTreatment};
use bijux_dna_core::prelude::id_catalog;

use super::profile_contracts::{fastq_capabilities, fastq_library_model};
use crate::fastq::defaults::{
    amplicon_standard_required_stages, amplicon_umi_required_stages,
    contaminant_depletion_required_stages, default_shotgun_required_stages,
    edna_metabarcoding_required_stages, generic_fastq_defaults, host_depletion_required_stages,
    qc_only_required_stages, rrna_depletion_required_stages, umi_required_stages,
};
use crate::{PipelineId, PipelineProfile, StabilityTier};

#[must_use]
pub fn fastq_qc_only_profile() -> PipelineProfile {
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_QC_ONLY),
        description: "FASTQ QC-only production profile",
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
        capabilities: fastq_capabilities(
            id_catalog::PIPELINE_FASTQ_QC_ONLY,
            qc_only_required_stages(),
        ),
    }
}

#[must_use]
pub fn fastq_trim_qc_profile() -> PipelineProfile {
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_TRIM_QC),
        description: "FASTQ trim-and-QC production profile",
        stability: StabilityTier::Stable,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults: generic_fastq_defaults(),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: fastq_library_model(
            LibraryLayout::PairedEnd,
            UdgTreatment::Unknown,
            AssayKind::Shotgun,
        ),
        capabilities: fastq_capabilities(
            id_catalog::PIPELINE_FASTQ_TRIM_QC,
            default_shotgun_required_stages(),
        ),
    }
}

#[must_use]
pub fn fastq_umi_profile() -> PipelineProfile {
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_UMI),
        description: "FASTQ UMI-aware production profile",
        stability: StabilityTier::Beta,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults: generic_fastq_defaults(),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: fastq_library_model(
            LibraryLayout::PairedEnd,
            UdgTreatment::Unknown,
            AssayKind::Shotgun,
        ),
        capabilities: fastq_capabilities(id_catalog::PIPELINE_FASTQ_UMI, umi_required_stages()),
    }
}

#[must_use]
pub fn fastq_host_depletion_profile() -> PipelineProfile {
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_HOST_DEPLETION),
        description: "FASTQ host-depletion production profile",
        stability: StabilityTier::Beta,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults: generic_fastq_defaults(),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: fastq_library_model(
            LibraryLayout::PairedEnd,
            UdgTreatment::Unknown,
            AssayKind::Shotgun,
        ),
        capabilities: fastq_capabilities(
            id_catalog::PIPELINE_FASTQ_HOST_DEPLETION,
            host_depletion_required_stages(),
        ),
    }
}

#[must_use]
pub fn fastq_rrna_depletion_profile() -> PipelineProfile {
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_RRNA_DEPLETION),
        description: "FASTQ rRNA-depletion production profile",
        stability: StabilityTier::Beta,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults: generic_fastq_defaults(),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: fastq_library_model(
            LibraryLayout::PairedEnd,
            UdgTreatment::Unknown,
            AssayKind::Shotgun,
        ),
        capabilities: fastq_capabilities(
            id_catalog::PIPELINE_FASTQ_RRNA_DEPLETION,
            rrna_depletion_required_stages(),
        ),
    }
}

#[must_use]
pub fn fastq_contaminant_depletion_profile() -> PipelineProfile {
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_CONTAMINANT_DEPLETION),
        description: "FASTQ contaminant-depletion production profile",
        stability: StabilityTier::Beta,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults: generic_fastq_defaults(),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: fastq_library_model(
            LibraryLayout::PairedEnd,
            UdgTreatment::Unknown,
            AssayKind::Shotgun,
        ),
        capabilities: fastq_capabilities(
            id_catalog::PIPELINE_FASTQ_CONTAMINANT_DEPLETION,
            contaminant_depletion_required_stages(),
        ),
    }
}

#[must_use]
pub fn fastq_amplicon_standard_profile() -> PipelineProfile {
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_AMPLICON_STANDARD),
        description: "FASTQ amplicon production profile",
        stability: StabilityTier::Beta,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults: generic_fastq_defaults(),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: fastq_library_model(
            LibraryLayout::PairedEnd,
            UdgTreatment::Unknown,
            AssayKind::Amplicon,
        ),
        capabilities: fastq_capabilities(
            id_catalog::PIPELINE_FASTQ_AMPLICON_STANDARD,
            amplicon_standard_required_stages(),
        ),
    }
}

#[must_use]
pub fn fastq_amplicon_umi_profile() -> PipelineProfile {
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_AMPLICON_UMI),
        description: "FASTQ amplicon UMI production profile",
        stability: StabilityTier::Beta,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults: generic_fastq_defaults(),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: fastq_library_model(
            LibraryLayout::PairedEnd,
            UdgTreatment::Unknown,
            AssayKind::Amplicon,
        ),
        capabilities: fastq_capabilities(
            id_catalog::PIPELINE_FASTQ_AMPLICON_UMI,
            amplicon_umi_required_stages(),
        ),
    }
}

#[must_use]
pub fn fastq_edna_metabarcoding_profile() -> PipelineProfile {
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_EDNA_METABARCODING),
        description: "FASTQ eDNA/metabarcoding production profile",
        stability: StabilityTier::Beta,
        input_domains: vec![crate::Domain::Fastq],
        output_domains: vec![crate::Domain::Fastq],
        defaults: generic_fastq_defaults(),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: fastq_library_model(
            LibraryLayout::PairedEnd,
            UdgTreatment::Unknown,
            AssayKind::Amplicon,
        ),
        capabilities: fastq_capabilities(
            id_catalog::PIPELINE_FASTQ_EDNA_METABARCODING,
            edna_metabarcoding_required_stages(),
        ),
    }
}
