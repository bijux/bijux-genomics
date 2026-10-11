use super::model::{
    ReportQcContributorArtifact, ReportQcContributorContract, REPORT_QC_CONTRIBUTORS,
};
use crate::campaign::paths::benchmark_sample_root;
use crate::catalog::corpus_fastq_stage_catalog_entry as stage_command_spec;
use crate::ports::CampaignPaths;
use anyhow::Result;
use std::path::PathBuf;
pub(crate) fn report_qc_required_contributor_artifacts(
    workspace_config: &dyn CampaignPaths,
    corpus_id: &str,
    sample_id: &str,
) -> Result<Vec<ReportQcContributorArtifact>> {
    REPORT_QC_CONTRIBUTORS
        .iter()
        .copied()
        .map(|contract| {
            resolve_report_qc_contributor_artifact(workspace_config, corpus_id, sample_id, contract)
        })
        .collect()
}

pub(crate) fn resolve_report_qc_contributor_artifact(
    workspace_config: &dyn CampaignPaths,
    corpus_id: &str,
    sample_id: &str,
    contract: ReportQcContributorContract,
) -> Result<ReportQcContributorArtifact> {
    for tool_id in contract.candidate_tool_ids {
        let path = report_qc_contributor_artifact_path(
            workspace_config,
            corpus_id,
            sample_id,
            contract.stage_id,
            tool_id,
            contract.relative_path,
        )?;
        if path.exists() {
            return Ok(ReportQcContributorArtifact {
                contract,
                tool_id: (*tool_id).to_string(),
                path,
            });
        }
    }

    let default_path = report_qc_contributor_artifact_path(
        workspace_config,
        corpus_id,
        sample_id,
        contract.stage_id,
        contract.default_tool_id,
        contract.relative_path,
    )?;
    Ok(ReportQcContributorArtifact {
        contract,
        tool_id: contract.default_tool_id.to_string(),
        path: default_path,
    })
}

pub(crate) fn report_qc_contributor_artifact_path(
    workspace_config: &dyn CampaignPaths,
    corpus_id: &str,
    sample_id: &str,
    stage_id: &str,
    tool_id: &str,
    relative_path: &str,
) -> Result<PathBuf> {
    let stage_root = workspace_config.stage_out_root(corpus_id, stage_id)?;
    let stage_spec = stage_command_spec(stage_id)?;
    Ok(benchmark_sample_root(&stage_root, stage_spec.report_dir, sample_id)
        .join("tools")
        .join(tool_id)
        .join(relative_path))
}
