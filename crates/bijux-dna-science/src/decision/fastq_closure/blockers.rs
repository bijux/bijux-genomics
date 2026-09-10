//! Admission reasons for closure; declarations alone do not establish readiness.
use crate::domain::FastqEnvironmentRow;
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn closure_blockers(
    row: &FastqEnvironmentRow,
    download_by_tool: &BTreeMap<&str, &crate::domain::FastqDownloadBacklogRow>,
    paper_by_tool: &BTreeMap<&str, Vec<&crate::domain::FastqPaperArchiveRow>>,
    qa_blockers_by_stage: &BTreeMap<&str, BTreeSet<&str>>,
) -> Vec<String> {
    let mut blockers = Vec::<String>::new();
    add_environment_blockers(row, &mut blockers);
    add_download_blockers(download_by_tool.get(row.tool_id.as_str()).copied(), &mut blockers);
    add_paper_blockers(row, paper_by_tool, &mut blockers);
    if row.benchmark_support == "none" {
        blockers.push("missing_benchmark_support".to_string());
    }
    if let Some(stage_qa_blockers) = qa_blockers_by_stage.get(row.stage_id.as_str()) {
        blockers.extend(stage_qa_blockers.iter().map(|blocker| (*blocker).to_string()));
    }
    blockers
}

fn add_environment_blockers(row: &FastqEnvironmentRow, blockers: &mut Vec<String>) {
    if row.execution_status != "closed" {
        blockers.push("stage_not_marked_closed".to_string());
    }
    if row.registry_status != "production" {
        blockers.push("registry_not_production".to_string());
    }
    if row.container_ref.trim().is_empty() {
        blockers.push("missing_container_ref".to_string());
    }
    if row.container_ref.contains("sha256:pending") {
        blockers.push("pending_container_digest".to_string());
    }
    if row
        .container_ref
        .contains("@sha256:0000000000000000000000000000000000000000000000000000000000000000")
    {
        blockers.push("placeholder_container_digest".to_string());
    }
    if row.runtimes.trim().is_empty() {
        blockers.push("missing_runtime_surface".to_string());
    }
}

fn add_download_blockers(
    download: Option<&crate::domain::FastqDownloadBacklogRow>,
    blockers: &mut Vec<String>,
) {
    match download {
        Some(download) => {
            if download.backlog_status != "ready" {
                blockers.push(format!("source_backlog_{}", download.backlog_status));
            }
            if download.archive_status == "missing" {
                blockers.push("missing_upstream_archive".to_string());
            }
            if matches!(download.paper_status.as_str(), "missing_paper_root" | "missing_paper_map")
            {
                blockers.push(download.paper_status.clone());
            }
        }
        None => blockers.push("missing_download_backlog_row".to_string()),
    }
}

fn add_paper_blockers(
    row: &FastqEnvironmentRow,
    paper_by_tool: &BTreeMap<&str, Vec<&crate::domain::FastqPaperArchiveRow>>,
    blockers: &mut Vec<String>,
) {
    let has_present_paper = paper_by_tool
        .get(row.tool_id.as_str())
        .is_some_and(|rows| rows.iter().any(|paper| paper.archive_status == "present"));
    if !has_present_paper {
        blockers.push("missing_paper_archive".to_string());
    }
}

pub(super) fn closure_warnings(row: &FastqEnvironmentRow) -> Vec<String> {
    let mut warnings = Vec::new();
    if row.tool_status == "disallowed" {
        warnings.push("planned_binding_not_admitted".to_string());
    }
    if !row.is_default {
        warnings.push("non_default_binding".to_string());
    }
    warnings
}
