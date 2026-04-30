use crate::vcf::workflow_registry::vcf_workflow_templates_for_pipeline;
use crate::{ArtifactType, Domain, MetricsBundle, PipelineCapabilities, ReportSection};

pub(super) fn vcf_capabilities(pipeline_id: &str) -> PipelineCapabilities {
    let required_stages =
        vec!["vcf.call".to_string(), "vcf.filter".to_string(), "vcf.stats".to_string()];
    let templates = vcf_workflow_templates_for_pipeline(pipeline_id);
    let primary_template = templates.first().cloned();

    PipelineCapabilities {
        input_domains: vec![Domain::Vcf],
        output_domains: vec![Domain::Vcf],
        input_artifacts: vec![ArtifactType::ReportJson],
        output_artifacts: vec![ArtifactType::ReportJson, ArtifactType::MetricsBundle],
        required_inputs: vec!["vcf", "sample_name"],
        produces_outputs: vec!["vcf", "vcf.metrics"],
        report_sections: vec!["vcf"],
        required_report_sections: vec![ReportSection::Vcf, ReportSection::PipelineDefaults],
        required_metrics_bundles: vec![MetricsBundle::VcfCore],
        required_stages,
        required_metrics: vec!["vcf.metrics"],
        required_artifacts: vec![
            "report.json",
            "run_manifest.json",
            "tool_provenance.json",
            "invariants_report.json",
            "vcf.tbi",
        ],
        supports_benchmarks: false,
        supports_sample_sheet: false,
        workflow_template_ids: templates
            .iter()
            .map(|template| template.template_id.clone())
            .collect(),
        batch_semantics: primary_template.as_ref().map(|template| template.batch_semantics.clone()),
        fan_artifact_rules: primary_template
            .as_ref()
            .map_or_else(Vec::new, |template| template.fan_artifact_rules.clone()),
        failure_policy: primary_template
            .as_ref()
            .map_or_else(Vec::new, |template| template.failure_policy.clone()),
        evidence_summary: primary_template
            .as_ref()
            .map(|template| template.evidence_summary.clone()),
        parameter_policy: primary_template
            .as_ref()
            .map(|template| template.parameter_policy.clone()),
    }
}
