use super::contracts::ensure_fastq_report_map_contract;
use super::inventory::admission_support_status;
use super::inventory::{collect_report_stage_observations, ReportStageObservations};
use super::models::FastqReportMapRow;
use crate::readiness::declarations::observations::ExpectedResultBinding;
use crate::readiness::declarations::ReportContext;
use anyhow::anyhow;
use anyhow::Result;
/// Collect canonical FASTQ declaration evidence.
///
/// # Errors
/// Returns source admission, contract or publication failure.
pub fn collect_fastq_report_map_rows(
    context: &ReportContext<'_>,
) -> Result<Vec<FastqReportMapRow>> {
    let repo_root = context.repo_root;
    let observations = collect_report_stage_observations(context)?;
    let expected_rows = context
        .evidence
        .expected(repo_root)?
        .into_iter()
        .filter(|row| row.domain == "fastq")
        .collect::<Vec<_>>();
    project_report_map_rows(&observations, &expected_rows)
}

pub(super) fn collect_report_map_rows_for_expected(
    context: &ReportContext<'_>,
    expected_rows: &[ExpectedResultBinding],
) -> Result<Vec<FastqReportMapRow>> {
    let observations = collect_report_stage_observations(context)?;
    project_report_map_rows(&observations, expected_rows)
}

fn project_report_map_rows(
    ReportStageObservations { metadata: stage_metadata, admissions: stage_admissions }: &ReportStageObservations,
    expected_rows: &[ExpectedResultBinding],
) -> Result<Vec<FastqReportMapRow>> {
    let mut rows = Vec::new();

    for row in expected_rows {
        let metadata = stage_metadata.get(&row.stage_id).ok_or_else(|| {
            anyhow!(
                "FASTQ report map is missing stage metadata for expected result `{}` / `{}`",
                row.stage_id,
                row.tool_id
            )
        })?;
        let admissions = stage_admissions.get(row.stage_id.as_str()).ok_or_else(|| {
            anyhow!("FASTQ report map is missing admitted benchmark tools for `{}`", row.stage_id)
        })?;
        let support_status = admission_support_status(admissions, row.tool_id.as_str())?;
        let stage_id = row.stage_id.clone();
        let tool_id = row.tool_id.clone();

        rows.push(FastqReportMapRow {
            result_row_id: row.result_row_id.clone(),
            stage_id: stage_id.clone(),
            tool_id: tool_id.clone(),
            corpus_family_id: row.corpus_family_id.clone(),
            fixture_id: row.fixture_id.clone(),
            sample_scope: row.sample_scope.clone(),
            canonical_stage_rank: metadata.canonical_stage_rank,
            readiness_kind: metadata.readiness_kind.clone(),
            stage_kind: metadata.stage_kind.clone(),
            criticality: metadata.criticality.clone(),
            support_status: support_status.to_string(),
            report_section_id: metadata.report_section_id.clone(),
            report_section_title: metadata.report_section_title.clone(),
            summary_table_id: metadata.summary_table_id.clone(),
            summary_table_title: metadata.summary_table_title.clone(),
            metric_classes: metadata.metric_classes.clone(),
            mutates_fastq: metadata.mutates_fastq,
            produces_reports_only: metadata.produces_reports_only,
            report_focus: metadata.report_focus.clone(),
            reason: report_reason(
                stage_id.as_str(),
                tool_id.as_str(),
                metadata.report_section_id.as_str(),
                metadata.summary_table_id.as_str(),
                metadata.report_focus.as_str(),
                support_status,
                metadata.produces_reports_only,
            ),
        });
    }

    rows.sort_by(|left, right| {
        left.canonical_stage_rank
            .cmp(&right.canonical_stage_rank)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_fastq_report_map_contract(&rows, expected_rows)?;
    Ok(rows)
}

pub(super) fn report_reason(
    stage_id: &str,
    tool_id: &str,
    report_section_id: &str,
    summary_table_id: &str,
    report_focus: &str,
    support_status: &str,
    produces_reports_only: bool,
) -> String {
    let stage_mode = if produces_reports_only { "reports-only" } else { "transforming" };
    format!(
        "benchmark result binding `{stage_id}` / `{tool_id}` is a FASTQ {stage_mode} row with governed support status `{support_status}`; benchmark reporting places it in section `{report_section_id}` and summary table `{summary_table_id}` to track {report_focus}"
    )
}
