use super::{
    input::read_json, observations::LocalKinshipSmokeCaseReport, paths::ensure_repo_relative_file,
};
use anyhow::{Context, Result};
use std::{fs, path::Path};
pub(super) struct CaseDocuments {
    pub ready_report: serde_json::Value,
    pub ready_summary: serde_json::Value,
    pub ready_stage_metrics: serde_json::Value,
    pub insufficient_report: serde_json::Value,
    pub insufficient_summary: serde_json::Value,
    pub insufficient_stage_metrics: serde_json::Value,
    pub ready_segments: String,
}
pub(super) fn read_case_documents(
    repo_root: &Path,
    ready_case: &LocalKinshipSmokeCaseReport,
    insufficient_case: &LocalKinshipSmokeCaseReport,
) -> Result<CaseDocuments> {
    ensure_repo_relative_file(repo_root, &ready_case.kinship_report)?;
    ensure_repo_relative_file(repo_root, &ready_case.kinship_summary)?;
    ensure_repo_relative_file(repo_root, &ready_case.kinship_segments)?;
    ensure_repo_relative_file(repo_root, &ready_case.stage_metrics)?;
    ensure_repo_relative_file(repo_root, &insufficient_case.kinship_report)?;
    ensure_repo_relative_file(repo_root, &insufficient_case.kinship_summary)?;
    ensure_repo_relative_file(repo_root, &insufficient_case.kinship_segments)?;
    ensure_repo_relative_file(repo_root, &insufficient_case.stage_metrics)?;

    let ready_report = read_json(&repo_root.join(&ready_case.kinship_report))?;
    let ready_summary = read_json(&repo_root.join(&ready_case.kinship_summary))?;
    let ready_stage_metrics = read_json(&repo_root.join(&ready_case.stage_metrics))?;
    let insufficient_report = read_json(&repo_root.join(&insufficient_case.kinship_report))?;
    let insufficient_summary = read_json(&repo_root.join(&insufficient_case.kinship_summary))?;
    let insufficient_stage_metrics = read_json(&repo_root.join(&insufficient_case.stage_metrics))?;
    let ready_segments = fs::read_to_string(repo_root.join(&ready_case.kinship_segments))
        .with_context(|| {
            format!("read {}", repo_root.join(&ready_case.kinship_segments).display())
        })?;

    Ok(CaseDocuments {
        ready_report,
        ready_summary,
        ready_stage_metrics,
        insufficient_report,
        insufficient_summary,
        insufficient_stage_metrics,
        ready_segments,
    })
}
