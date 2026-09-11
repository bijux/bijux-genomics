use super::model::{
    ReportQcContributorArtifact, REPORT_QC_CONTRIBUTORS, REPORT_QC_UPSTREAM_STAGES,
};
use std::collections::BTreeSet;
pub(crate) fn report_qc_contributor_id(artifact: &ReportQcContributorArtifact) -> String {
    format!("{}.{}", artifact.contract.stage_id, artifact.tool_id)
}

pub(crate) fn report_qc_artifact_name(artifact: &ReportQcContributorArtifact) -> String {
    format!(
        "{}.tool.{}.{}",
        artifact.contract.stage_id, artifact.tool_id, artifact.contract.artifact_id
    )
}

pub(crate) fn report_qc_contributor_tool_ids() -> Vec<String> {
    REPORT_QC_CONTRIBUTORS
        .iter()
        .flat_map(|row| row.candidate_tool_ids.iter().copied())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(crate) fn report_qc_upstream_stage_ids() -> Vec<String> {
    REPORT_QC_UPSTREAM_STAGES.iter().map(|row| row.stage_id.to_string()).collect()
}
