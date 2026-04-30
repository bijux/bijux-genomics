use bijux_dna_core::ids::{AssayKind, LibraryLayout, LibraryModel, PlatformHint, StageId, ToolId, UdgTreatment};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::defaults::default_params_json;
use bijux_dna_domain_bam::BamStage;

use crate::cross::workflow_registry::cross_workflow_templates_for_pipeline;
use crate::cross::bam_to_vcf::merged_defaults::default_base_defaults;
use crate::cross::bam_to_vcf::required_stages::required_cross_stages;
use crate::{
    ArtifactType, DefaultParams, Domain, EmptyParams, MetricsBundle, PipelineCapabilities,
    PipelineId, PipelineProfile, ReportSection, StabilityTier,
};

#[must_use]
pub fn bam_to_vcf_default_profile() -> PipelineProfile {
    let (bam_profile, vcf_profile, mut defaults) = default_base_defaults();
    let required_stages = required_cross_stages(&bam_profile);
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
        "cross-domain reference preparation before BAM-to-VCF calling".to_string(),
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
    defaults.rationales.insert(
        StageId::from_static("bam.genotyping"),
        "cross-domain BAM-to-VCF calling defaults".to_string(),
    );
    for stage_id in [id_catalog::VCF_FILTER, id_catalog::VCF_STATS] {
        let stage = StageId::from_static(stage_id);
        if let Some(tool) = vcf_profile.defaults.tools.get(&stage).cloned() {
            defaults.tools.insert(stage.clone(), tool);
        }
        if let Some(params) = vcf_profile.defaults.params.get(&stage).cloned() {
            defaults.params.insert(stage.clone(), params);
        }
        if let Some(rationale) = vcf_profile.defaults.rationales.get(&stage).cloned() {
            defaults.rationales.insert(stage, rationale);
        }
    }
    let template_ids = cross_workflow_templates_for_pipeline(id_catalog::PIPELINE_BAM_TO_VCF_DEFAULT)
        .into_iter()
        .map(|template| template.template_id)
        .collect::<Vec<_>>();
    let template = cross_workflow_templates_for_pipeline(id_catalog::PIPELINE_BAM_TO_VCF_DEFAULT)
        .into_iter()
        .next()
        .expect("cross workflow template must exist for bam-to-vcf default");

    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_BAM_TO_VCF_DEFAULT),
        description: "BAM QC and genotyping handoff into VCF filtering and stats",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Bam, Domain::Cross],
        output_domains: vec![Domain::Vcf],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: LibraryModel {
            layout: LibraryLayout::SingleEnd,
            udg_treatment: UdgTreatment::Unknown,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Unknown,
        },
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Bam, Domain::Cross],
            output_domains: vec![Domain::Vcf],
            input_artifacts: vec![
                ArtifactType::Bam,
                ArtifactType::ReferenceFasta,
                ArtifactType::SampleSheet,
            ],
            output_artifacts: vec![ArtifactType::Variant, ArtifactType::MetricsBundle],
            required_inputs: vec!["bam", "bam_index", "reference", "sample_name"],
            produces_outputs: vec!["bam.metrics", "vcf", "vcf.metrics"],
            report_sections: vec!["bam", "vcf", "cross.handoff"],
            required_report_sections: vec![
                ReportSection::Bam,
                ReportSection::Vcf,
                ReportSection::Handoff,
                ReportSection::PipelineDefaults,
            ],
            required_metrics_bundles: vec![
                MetricsBundle::BamCore,
                MetricsBundle::VcfCore,
                MetricsBundle::CrossHandoff,
            ],
            required_stages,
            required_metrics: vec!["bam.metrics", "vcf.metrics"],
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
