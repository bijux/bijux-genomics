use bijux_dna_core::ids::{
    AssayKind, LibraryLayout, LibraryModel, PlatformHint, StageId, ToolId, UdgTreatment,
};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::defaults::adna_shotgun_params_json;
use bijux_dna_domain_bam::BamStage;

use crate::cross::fastq_to_bam::merged_defaults::adna_base_defaults;
use crate::cross::fastq_to_bam::required_stages::required_cross_stages;
use crate::{
    ArtifactType, DefaultParams, Domain, EmptyParams, InvariantsPreset, MetricsBundle,
    PipelineCapabilities, PipelineId, PipelineProfile, ReportSection, StabilityTier,
};

#[must_use]
pub fn fastq_to_bam_adna_shotgun_profile() -> PipelineProfile {
    let (fastq_profile, _bam_profile, mut defaults) = adna_base_defaults();
    let required_stages = required_cross_stages(&fastq_profile);
    defaults.tools.insert(
        StageId::from_static(id_catalog::CORE_PREPARE_REFERENCE),
        ToolId::from_static(id_catalog::TOOL_SAMTOOLS),
    );
    defaults.params.insert(
        StageId::from_static(id_catalog::CORE_PREPARE_REFERENCE),
        DefaultParams::Empty(EmptyParams::default()),
    );
    defaults.rationales.insert(
        StageId::from_static(id_catalog::CORE_PREPARE_REFERENCE),
        "reference prep uses canonical defaults for cross-domain alignment".to_string(),
    );
    defaults.params.insert(
        StageId::from_static(id_catalog::BAM_ALIGN),
        DefaultParams::Bam(
            BamStage::Align
                .parse_effective_params(&adna_shotgun_params_json(BamStage::Align))
                .unwrap_or_else(|err| {
                    panic!(
                        "failed to parse typed BAM defaults for stage {}: {err}",
                        BamStage::Align.as_str()
                    )
                }),
        ),
    );
    defaults.tools.insert(
        StageId::from_static(id_catalog::BAM_ALIGN),
        ToolId::from_static(id_catalog::TOOL_BWA),
    );
    defaults.rationales.insert(
        StageId::from_static(id_catalog::BAM_ALIGN),
        "aDNA default alignment preset".to_string(),
    );
    defaults.params.insert(
        StageId::from_static("bam.qc_pre"),
        DefaultParams::Bam(
            BamStage::QcPre
                .parse_effective_params(&adna_shotgun_params_json(BamStage::QcPre))
                .unwrap_or_else(|err| {
                    panic!(
                        "failed to parse typed BAM defaults for stage {}: {err}",
                        BamStage::QcPre.as_str()
                    )
                }),
        ),
    );
    defaults
        .tools
        .insert(StageId::from_static("bam.qc_pre"), ToolId::from_static(id_catalog::TOOL_SAMTOOLS));
    defaults.rationales.insert(
        StageId::from_static("bam.qc_pre"),
        "cross-domain compatibility bridge for BAM pre-QC defaults".to_string(),
    );

    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_TO_BAM_ADNA_SHOTGUN),
        description: "FASTQ preprocess -> align -> BAM QC/damage (aDNA shotgun)",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Fastq, Domain::Cross],
        output_domains: vec![Domain::Bam],
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
            input_domains: vec![Domain::Fastq, Domain::Cross],
            output_domains: vec![Domain::Bam],
            input_artifacts: vec![ArtifactType::FastqReads, ArtifactType::ReferenceFasta],
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
            required_stages,
            required_metrics: vec!["fastq.metrics", "bam.metrics"],
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "stage_summaries.json",
                "invariants_report.json",
            ],
            supports_benchmarks: false,
            supports_sample_sheet: false,
            workflow_template_ids: Vec::new(),
            batch_semantics: None,
            fan_artifact_rules: Vec::new(),
            failure_policy: Vec::new(),
            evidence_summary: None,
            parameter_policy: None,
        },
    }
}
