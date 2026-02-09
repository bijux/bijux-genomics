//! Cross-domain FASTQ → BAM profile (aDNA).

use crate::bam::bam_adna_shotgun_profile;
use crate::fastq::fastq_default_profile;
use crate::{
    ArtifactType, DefaultParams, Domain, EffectiveDefaults, MetricsBundle, PipelineCapabilities,
    PipelineId, PipelineProfile, ReportSection, StabilityTier,
};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::defaults::{adna_shotgun_params_json, default_params_json};
use bijux_dna_domain_bam::BamStage;

fn base_defaults() -> (PipelineProfile, PipelineProfile, EffectiveDefaults) {
    let fastq_profile = fastq_default_profile();
    let bam_profile = bam_adna_shotgun_profile();

    let mut defaults = EffectiveDefaults::default();
    defaults.tools.extend(fastq_profile.defaults.tools.clone());
    defaults
        .params
        .extend(fastq_profile.defaults.params.clone());
    defaults
        .rationales
        .extend(fastq_profile.defaults.rationales.clone());
    defaults.tools.extend(bam_profile.defaults.tools.clone());
    defaults.params.extend(bam_profile.defaults.params.clone());
    defaults
        .rationales
        .extend(bam_profile.defaults.rationales.clone());
    (fastq_profile, bam_profile, defaults)
}

#[must_use]
pub fn fastq_to_bam_adna_shotgun_profile() -> PipelineProfile {
    let (_fastq_profile, _bam_profile, mut defaults) = base_defaults();
    defaults.tools.insert(
        StageId::from_static("core.prepare_reference"),
        ToolId::from_static("samtools"),
    );
    defaults.params.insert(
        StageId::from_static("core.prepare_reference"),
        DefaultParams::Json(serde_json::json!({})),
    );
    defaults.rationales.insert(
        StageId::from_static("core.prepare_reference"),
        "reference prep uses canonical defaults for cross-domain alignment".to_string(),
    );
    defaults.params.insert(
        StageId::from_static("bam.align"),
        DefaultParams::Json(adna_shotgun_params_json(BamStage::Align)),
    );
    defaults.tools.insert(
        StageId::from_static("bam.align"),
        ToolId::from_static("bwa"),
    );
    defaults.rationales.insert(
        StageId::from_static("bam.align"),
        "aDNA default alignment preset".to_string(),
    );

    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_TO_BAM_ADNA_SHOTGUN),
        description: "FASTQ preprocess → align → BAM QC/damage (aDNA shotgun)",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Fastq, Domain::Cross],
        output_domains: vec![Domain::Bam],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some("adna"),
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Fastq, Domain::Cross],
            output_domains: vec![Domain::Bam],
            input_artifacts: vec![ArtifactType::FastqReads, ArtifactType::Bam],
            output_artifacts: vec![ArtifactType::Bam, ArtifactType::MetricsBundle],
            required_inputs: vec!["fastq", "reference"],
            produces_outputs: vec!["fastq", "bam", "bam.metrics"],
            report_sections: vec!["fastq", "bam", "cross.handoff"],
            required_report_sections: vec![
                ReportSection::Fastq,
                ReportSection::Bam,
                ReportSection::Handoff,
                ReportSection::PipelineDefaults,
            ],
            required_metrics_bundles: vec![MetricsBundle::FastqCore, MetricsBundle::BamAdna],
            required_stages: vec![
                "fastq.preprocess",
                "core.prepare_reference",
                "bam.align",
                "bam.qc_pre",
                "bam.coverage",
                "bam.damage",
            ],
            required_metrics: vec!["fastq.metrics", "bam.metrics"],
            required_artifacts: vec!["report.json", "run_manifest.json", "stage_summaries.json"],
            supports_benchmarks: false,
        },
    }
}

#[must_use]
pub fn fastq_to_bam_default_profile() -> PipelineProfile {
    let (_fastq_profile, _bam_profile, mut defaults) = base_defaults();
    defaults.tools.insert(
        StageId::from_static("core.prepare_reference"),
        ToolId::from_static("samtools"),
    );
    defaults.params.insert(
        StageId::from_static("core.prepare_reference"),
        DefaultParams::Json(serde_json::json!({})),
    );
    defaults.rationales.insert(
        StageId::from_static("core.prepare_reference"),
        "reference prep uses canonical defaults for cross-domain alignment".to_string(),
    );
    defaults.params.insert(
        StageId::from_static("bam.align"),
        DefaultParams::Json(default_params_json(BamStage::Align)),
    );
    defaults.tools.insert(
        StageId::from_static("bam.align"),
        ToolId::from_static("bwa"),
    );
    defaults.rationales.insert(
        StageId::from_static("bam.align"),
        "modern default alignment preset".to_string(),
    );

    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_TO_BAM_DEFAULT),
        description: "FASTQ preprocess → align → BAM QC/damage (modern defaults)",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Fastq, Domain::Cross],
        output_domains: vec![Domain::Bam],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Fastq, Domain::Cross],
            output_domains: vec![Domain::Bam],
            input_artifacts: vec![ArtifactType::FastqReads, ArtifactType::Bam],
            output_artifacts: vec![ArtifactType::Bam, ArtifactType::MetricsBundle],
            required_inputs: vec!["fastq", "reference"],
            produces_outputs: vec!["fastq", "bam", "bam.metrics"],
            report_sections: vec!["fastq", "bam", "cross.handoff"],
            required_report_sections: vec![
                ReportSection::Fastq,
                ReportSection::Bam,
                ReportSection::Handoff,
                ReportSection::PipelineDefaults,
            ],
            required_metrics_bundles: vec![MetricsBundle::FastqCore, MetricsBundle::BamCore],
            required_stages: vec![
                "fastq.preprocess",
                "core.prepare_reference",
                "bam.align",
                "bam.qc_pre",
                "bam.coverage",
                "bam.damage",
            ],
            required_metrics: vec!["fastq.metrics", "bam.metrics"],
            required_artifacts: vec!["report.json", "run_manifest.json", "stage_summaries.json"],
            supports_benchmarks: false,
        },
    }
}
