use crate::{ArtifactType, Domain, MetricsBundle, PipelineCapabilities, ReportSection};

pub(crate) fn fastq_capabilities(required_stages: Vec<String>) -> PipelineCapabilities {
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
        workflow_template_ids: Vec::new(),
        batch_semantics: None,
        fan_artifact_rules: Vec::new(),
        failure_policy: Vec::new(),
        evidence_summary: None,
        parameter_policy: None,
    }
}
