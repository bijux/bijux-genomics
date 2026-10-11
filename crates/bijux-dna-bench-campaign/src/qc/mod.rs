//! Governed QC contributors, existing evidence selection and upstream preparation.
mod metadata;
mod model;
mod prepare;
mod resolve;
mod upstream;
const REPORT_QC_INPUTS_SCHEMA_VERSION: &str = "bijux.fastq.report_qc.inputs.v1";
#[derive(Debug, Default)]
pub(crate) struct StageSamplePreparation {
    pub(crate) extra_stage_args: Vec<String>,
    pub(crate) run_extra_fields: std::collections::BTreeMap<String, serde_json::Value>,
}
pub(crate) use metadata::{report_qc_contributor_tool_ids, report_qc_upstream_stage_ids};
pub(crate) use prepare::prepare_report_qc_sample;

#[cfg(test)]
mod contracts;

/// Execution and workspace context shared by QC preparation and its upstream stages.
#[derive(Clone, Copy)]
pub(crate) struct PreparationContext<'a> {
    pub program: &'a std::path::Path,
    pub repo_root: &'a std::path::Path,
    pub workspace_config: &'a dyn crate::ports::CampaignPaths,
    pub corpus_id: &'a str,
    pub platform: &'a str,
    pub dry_run: bool,
    pub executor: &'a dyn crate::ports::CampaignExecution,
}
