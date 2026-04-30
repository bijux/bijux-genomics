use bijux_dna_core::ids::{AssayKind, LibraryLayout, LibraryModel, PlatformHint, StageId, ToolId, UdgTreatment};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::defaults::default_params_json;
use bijux_dna_domain_bam::BamStage;

use crate::cross::fastq_to_vcf::merged_defaults::minimal_base_defaults;
use crate::cross::fastq_to_vcf::required_stages::required_cross_stages;
use crate::cross::workflow_registry::cross_workflow_templates_for_pipeline;
use crate::{
    ArtifactType, DefaultParams, Domain, EmptyParams, MetricsBundle, PipelineCapabilities,
    PipelineId, PipelineProfile, ReportSection, StabilityTier,
};

#[must_use]
pub fn fastq_to_vcf_minimal_profile() -> PipelineProfile {
    let (fastq_profile, _vcf_profile, mut defaults) = minimal_base_defaults();
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
        "cross-domain reference preparation for tiny FASTQ-to-VCF templates".to_string(),
    );
    defaults.tools.insert(
        StageId::from_static(id_catalog::BAM_ALIGN),
        ToolId::from_static(id_catalog::TOOL_BWA),
    );
    defaults.params.insert(
        StageId::from_static(id_catalog::BAM_ALIGN),
        DefaultParams::Bam(
            BamStage::Align
                .parse_effective_params(&default_params_json(BamStage::Align))
                .unwrap_or_else(|err| panic!("failed to parse BAM align defaults: {err}")),
        ),
    );
    defaults.tools.insert(
        StageId::from_static("bam.qc_pre"),
        ToolId::from_static(id_catalog::TOOL_SAMTOOLS),
    );
    defaults.params.insert(
        StageId::from_static("bam.qc_pre"),
        DefaultParams::Bam(
            BamStage::QcPre
                .parse_effective_params(&default_params_json(BamStage::QcPre))
                .unwrap_or_else(|err| panic!("failed to parse BAM qc_pre defaults: {err}")),
        ),
    );
    defaults.tools.insert(
        StageId::from_static("bam.genotyping"),
        ToolId::from_static(id_catalog::TOOL_ANGSD),
    );
    defaults.params.insert(
        StageId::from_static("bam.genotyping"),
        DefaultParams::Bam(
            BamStage::Genotyping
                .parse_effective_params(&default_params_json(BamStage::Genotyping))
                .unwrap_or_else(|err| panic!("failed to parse BAM genotyping defaults: {err}")),
        ),
    );
    let template_ids = cross_workflow_templates_for_pipeline(id_catalog::PIPELINE_FASTQ_TO_VCF_MINIMAL)
        .into_iter()
        .map(|template| template.template_id)
        .collect::<Vec<_>>();
    let template = cross_workflow_templates_for_pipeline(id_catalog::PIPELINE_FASTQ_TO_VCF_MINIMAL)
        .into_iter()
        .next()
        .expect("cross workflow template must exist for fastq-to-vcf minimal");

    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_TO_VCF_MINIMAL),
        description: "Minimal FASTQ preprocessing into BAM alignment and VCF calling/stats",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Fastq, Domain::Cross],
        output_domains: vec![Domain::Vcf],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: LibraryModel {
            layout: LibraryLayout::PairedEnd,
            udg_treatment: UdgTreatment::Unknown,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Shotgun,
        },
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Fastq, Domain::Cross],
            output_domains: vec![Domain::Vcf],
            input_artifacts: vec![
                ArtifactType::FastqReads,
                ArtifactType::ReferenceFasta,
                ArtifactType::SampleSheet,
            ],
            output_artifacts: vec![ArtifactType::Variant, ArtifactType::MetricsBundle],
            required_inputs: vec!["fastq", "reference", "sample_name"],
            produces_outputs: vec!["bam", "vcf", "vcf.metrics"],
            report_sections: vec!["fastq", "bam", "vcf", "cross.handoff"],
            required_report_sections: vec![
                ReportSection::Fastq,
                ReportSection::Bam,
                ReportSection::Vcf,
                ReportSection::Handoff,
                ReportSection::PipelineDefaults,
            ],
            required_metrics_bundles: vec![
                MetricsBundle::FastqCore,
                MetricsBundle::BamCore,
                MetricsBundle::VcfCore,
                MetricsBundle::CrossHandoff,
            ],
            required_stages,
            required_metrics: vec!["fastq.metrics", "bam.metrics", "vcf.metrics"],
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "stage_summaries.json",
                "invariants_report.json",
                "plan_manifest.json",
            ],
            supports_benchmarks: false,
            supports_sample_sheet: true,
            workflow_template_ids: template_ids,
            batch_semantics: Some(template.batch_semantics),
            fan_artifact_rules: template.fan_artifact_rules,
            failure_policy: template.failure_policy,
            evidence_summary: Some(template.evidence_summary),
            parameter_policy: Some(template.parameter_policy),
        },
    }
}
