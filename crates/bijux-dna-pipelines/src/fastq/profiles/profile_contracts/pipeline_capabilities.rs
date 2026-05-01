use crate::fastq::workflow_registry::fastq_workflow_templates_for_pipeline;
use crate::{ArtifactType, Domain, MetricsBundle, PipelineCapabilities, ReportSection};

pub(crate) fn fastq_capabilities(
    pipeline_id: &str,
    required_stages: Vec<String>,
) -> PipelineCapabilities {
    let templates = fastq_workflow_templates_for_pipeline(pipeline_id);
    let primary_template = templates.first().cloned();
    PipelineCapabilities {
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
