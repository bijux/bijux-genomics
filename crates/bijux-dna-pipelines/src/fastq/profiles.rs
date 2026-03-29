//! FASTQ pipeline profile definitions.

use bijux_dna_core::ids::{
    AssayKind, LibraryLayout, LibraryModel, PlatformHint, UdgTreatment,
};
use bijux_dna_core::prelude::id_catalog;

use super::defaults::{
    adna_fastq_defaults, append_stage_once, default_shotgun_required_stages, fastq_defaults,
    reference_adna_fastq_defaults,
};
use crate::{
    ArtifactType, Domain, InvariantsPreset, MetricsBundle, PipelineCapabilities, PipelineId,
    PipelineProfile, ReportSection, StabilityTier,
};

/// Stable FASTQ profile identifiers.
pub const FASTQ_PROFILE_IDS: &[&str] = &[
    "fastq-to-fastq__default__v1",
    "fastq-to-fastq__minimal__v1",
    "fastq-to-fastq__adna__v1",
    "fastq-to-fastq__reference_adna__v1",
];

#[must_use]
pub fn fastq_minimal_profile() -> PipelineProfile {
    let required_stages = default_shotgun_required_stages();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_MINIMAL),
        description: "Minimal FASTQ pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
        defaults: fastq_defaults(false),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: LibraryModel {
            layout: LibraryLayout::SingleEnd,
            udg_treatment: UdgTreatment::Unknown,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Unknown,
        },
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Fastq],
            output_domains: vec![Domain::Fastq],
            input_artifacts: vec![ArtifactType::FastqReads],
            output_artifacts: vec![ArtifactType::FastqReads, ArtifactType::MetricsBundle],
            required_inputs: vec!["fastq"],
            produces_outputs: vec!["fastq", "fastq.metrics"],
            report_sections: vec!["fastq"],
            required_report_sections: vec![ReportSection::Fastq, ReportSection::PipelineDefaults],
            required_metrics_bundles: vec![MetricsBundle::FastqCore],
            required_stages,
            required_metrics: vec!["fastq.metrics"],
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "stage_summaries.json",
                "invariants_report.json",
            ],
            supports_benchmarks: true,
        },
    }
}

#[must_use]
pub fn fastq_default_profile() -> PipelineProfile {
    let required_stages = default_shotgun_required_stages();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_DEFAULT),
        description: "Default FASTQ pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
        defaults: fastq_defaults(false),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: LibraryModel {
            layout: LibraryLayout::SingleEnd,
            udg_treatment: UdgTreatment::Unknown,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Unknown,
        },
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Fastq],
            output_domains: vec![Domain::Fastq],
            input_artifacts: vec![ArtifactType::FastqReads],
            output_artifacts: vec![ArtifactType::FastqReads, ArtifactType::MetricsBundle],
            required_inputs: vec!["fastq"],
            produces_outputs: vec!["fastq", "fastq.metrics"],
            report_sections: vec!["fastq"],
            required_report_sections: vec![ReportSection::Fastq, ReportSection::PipelineDefaults],
            required_metrics_bundles: vec![MetricsBundle::FastqCore],
            required_stages,
            required_metrics: vec!["fastq.metrics"],
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "stage_summaries.json",
                "invariants_report.json",
            ],
            supports_benchmarks: true,
        },
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
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some(InvariantsPreset::Adna),
        library_model: LibraryModel {
            layout: LibraryLayout::PairedEnd,
            udg_treatment: UdgTreatment::None,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Shotgun,
        },
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Fastq],
            output_domains: vec![Domain::Fastq],
            input_artifacts: vec![ArtifactType::FastqReads],
            output_artifacts: vec![ArtifactType::FastqReads, ArtifactType::MetricsBundle],
            required_inputs: vec!["fastq"],
            produces_outputs: vec!["fastq", "fastq.metrics"],
            report_sections: vec!["fastq"],
            required_report_sections: vec![ReportSection::Fastq, ReportSection::PipelineDefaults],
            required_metrics_bundles: vec![MetricsBundle::FastqCore],
            required_stages,
            required_metrics: vec!["fastq.metrics"],
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "stage_summaries.json",
                "invariants_report.json",
            ],
            supports_benchmarks: true,
        },
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
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some(InvariantsPreset::ReferenceAdna),
        library_model: LibraryModel {
            layout: LibraryLayout::PairedEnd,
            udg_treatment: UdgTreatment::None,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Shotgun,
        },
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Fastq],
            output_domains: vec![Domain::Fastq],
            input_artifacts: vec![ArtifactType::FastqReads],
            output_artifacts: vec![ArtifactType::FastqReads, ArtifactType::MetricsBundle],
            required_inputs: vec!["fastq"],
            produces_outputs: vec!["fastq", "fastq.metrics"],
            report_sections: vec!["fastq"],
            required_report_sections: vec![ReportSection::Fastq, ReportSection::PipelineDefaults],
            required_metrics_bundles: vec![MetricsBundle::FastqCore],
            required_stages,
            required_metrics: vec!["fastq.metrics"],
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "stage_summaries.json",
                "invariants_report.json",
            ],
            supports_benchmarks: true,
        },
    }
}

/// # Errors
/// Returns an error if the requested profile id is unknown.
pub fn fastq_profiles_by_id(id: &str) -> anyhow::Result<PipelineProfile> {
    match id {
        id_catalog::PIPELINE_FASTQ_DEFAULT => Ok(fastq_default_profile()),
        id_catalog::PIPELINE_FASTQ_MINIMAL => Ok(fastq_minimal_profile()),
        id_catalog::PIPELINE_FASTQ_ADNA => Ok(fastq_adna_profile()),
        id_catalog::PIPELINE_FASTQ_REFERENCE_ADNA => Ok(fastq_reference_adna_profile()),
        _ => Err(anyhow::anyhow!("unknown FASTQ profile: {id}")),
    }
}
