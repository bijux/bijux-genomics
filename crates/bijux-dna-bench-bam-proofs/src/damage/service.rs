use super::acquisition::load_damage_stage_proof;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_TOOL_IDS;
use super::models::BamDamageCompleteReport;
use super::report::project;
use super::report::ReportProjection;
use super::rows::build_bam_damage_complete_row;
use crate::ProofContext;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;
pub(super) fn build_bam_damage_complete_report(
    context: &ProofContext<'_>,
    output_path: &Path,
) -> Result<BamDamageCompleteReport> {
    let repo_root = context.repo_root;
    let readiness_report = context.readiness.damage_authenticity(repo_root)?;
    let proof = load_damage_stage_proof(context)?;

    let mut rows = readiness_report
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .map(|row| build_bam_damage_complete_row(repo_root, row, &proof))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));

    let expected_tool_ids =
        EXPECTED_TOOL_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let observed_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<Vec<_>>();
    let expected_tool_id_set = expected_tool_ids.iter().cloned().collect::<BTreeSet<_>>();
    let observed_tool_id_set = observed_tool_ids.iter().cloned().collect::<BTreeSet<_>>();
    let missing_tool_ids =
        expected_tool_id_set.difference(&observed_tool_id_set).cloned().collect::<Vec<_>>();
    let unexpected_tool_ids =
        observed_tool_id_set.difference(&expected_tool_id_set).cloned().collect::<Vec<_>>();
    let toolset_ready = missing_tool_ids.is_empty() && unexpected_tool_ids.is_empty();

    let complete_row_count = rows.iter().filter(|row| row.coverage_status == "complete").count();
    let violations =
        rows.iter().filter(|row| row.coverage_status != "complete").cloned().collect::<Vec<_>>();

    Ok(project(ReportProjection {
        output_path,
        repo_root,
        rows,
        expected_tool_ids,
        observed_tool_ids,
        missing_tool_ids,
        unexpected_tool_ids,
        toolset_ready,
        complete_row_count,
        violations,
    }))
}
