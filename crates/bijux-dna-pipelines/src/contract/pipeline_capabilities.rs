use serde::Serialize;

use super::{
    ArtifactType, BatchWorkflowSemanticsV1, CrossDomainEvidenceSummaryV1,
    CrossDomainFailurePolicyV1, FanArtifactRuleV1, MetricsBundle, ReportSection,
    TemplateParameterPolicyV1,
};

#[derive(Debug, Clone, Serialize)]
pub struct PipelineCapabilities {
    pub input_domains: Vec<super::Domain>,
    pub output_domains: Vec<super::Domain>,
    pub input_artifacts: Vec<ArtifactType>,
    pub output_artifacts: Vec<ArtifactType>,
    pub required_inputs: Vec<&'static str>,
    pub produces_outputs: Vec<&'static str>,
    pub report_sections: Vec<&'static str>,
    pub required_report_sections: Vec<ReportSection>,
    pub required_metrics_bundles: Vec<MetricsBundle>,
    pub required_stages: Vec<String>,
    pub required_metrics: Vec<&'static str>,
    pub required_artifacts: Vec<&'static str>,
    pub supports_benchmarks: bool,
    #[serde(default)]
    pub supports_sample_sheet: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workflow_template_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_semantics: Option<BatchWorkflowSemanticsV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fan_artifact_rules: Vec<FanArtifactRuleV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failure_policy: Vec<CrossDomainFailurePolicyV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_summary: Option<CrossDomainEvidenceSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_policy: Option<TemplateParameterPolicyV1>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineContract {
    pub pipeline_id: crate::PipelineId,
    pub required_stages: Vec<String>,
    pub required_artifacts: Vec<String>,
    pub required_metrics_bundles: Vec<MetricsBundle>,
    pub required_report_sections: Vec<ReportSection>,
}
