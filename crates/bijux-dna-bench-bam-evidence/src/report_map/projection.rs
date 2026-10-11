use super::{
    admission::ensure_bam_report_map_contract, anchors::admission_support_status,
    metadata::BamReportStageMetadata, models::BamReportMapRow, reason::report_reason,
};
use crate::observations::{ExpectedResultBinding, StageAdmissions};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
pub(super) fn project_rows(
    stage_metadata: &BTreeMap<String, BamReportStageMetadata>,
    stage_admissions: &StageAdmissions,
    expected_rows: &[ExpectedResultBinding],
) -> Result<Vec<BamReportMapRow>> {
    let mut rows = Vec::with_capacity(expected_rows.len());

    for row in expected_rows {
        let metadata = stage_metadata.get(&row.stage_id).ok_or_else(|| {
            anyhow!(
                "BAM report map is missing stage metadata for expected result `{}` / `{}`",
                row.stage_id,
                row.tool_id
            )
        })?;
        let admissions = stage_admissions.get(row.stage_id.as_str()).ok_or_else(|| {
            anyhow!("BAM report map is missing admitted benchmark tools for `{}`", row.stage_id)
        })?;
        let support_status = admission_support_status(admissions, row.tool_id.as_str())?;
        rows.push(BamReportMapRow {
            result_row_id: row.result_row_id.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_family_id: row.corpus_family_id.clone(),
            fixture_id: row.fixture_id.clone(),
            sample_scope: row.sample_scope.clone(),
            canonical_stage_rank: metadata.canonical_stage_rank,
            readiness_kind: metadata.readiness_kind.clone(),
            criticality: metadata.criticality.clone(),
            support_status: support_status.to_string(),
            anchor_tool_id: metadata.anchor_tool_id.clone(),
            anchor_support_status: metadata.anchor_support_status.clone(),
            report_section_id: metadata.report_section_id.clone(),
            report_section_title: metadata.report_section_title.clone(),
            summary_table_id: metadata.summary_table_id.clone(),
            summary_table_title: metadata.summary_table_title.clone(),
            workflow_branch_id: metadata.workflow_branch_id.clone(),
            scientific_context_required: metadata.scientific_context_required.clone(),
            report_focus: metadata.report_focus.clone(),
            reason: report_reason(row, metadata, support_status),
        });
    }

    rows.sort_by(|left, right| {
        left.canonical_stage_rank
            .cmp(&right.canonical_stage_rank)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.fixture_id.cmp(&right.fixture_id))
    });
    ensure_bam_report_map_contract(&rows, expected_rows)?;
    Ok(rows)
}
