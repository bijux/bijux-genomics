use super::declarations::EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_TOOL_ID;
use super::models::BamRecalibrationCompleteReport;
use super::observations::LocalRecalibrationSmokeReport;
use super::report::project;
use super::report::ReportProjection;
use super::rows::build_bam_recalibration_complete_row;
use crate::ports::NativeProofKind;
use crate::ProofContext;
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
pub(super) fn build_bam_recalibration_complete_report(
    context: &ProofContext<'_>,
    output_path: &Path,
) -> Result<BamRecalibrationCompleteReport> {
    let repo_root = context.repo_root;
    let readiness = context.readiness.recalibration_genotyping(repo_root)?;
    let smoke_path = context.native.write(repo_root, NativeProofKind::Recalibration)?;
    let smoke_report: LocalRecalibrationSmokeReport = crate::input::read_json(&smoke_path)?;

    if smoke_report.schema_version != EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected bam.recalibration local-smoke schema `{}`",
            smoke_report.schema_version
        ));
    }
    if smoke_report.stage_id != EXPECTED_STAGE_ID {
        return Err(anyhow!(
            "unexpected bam.recalibration local-smoke stage `{}`",
            smoke_report.stage_id
        ));
    }
    if smoke_report.tool_id != EXPECTED_TOOL_ID {
        return Err(anyhow!(
            "unexpected bam.recalibration local-smoke tool `{}`",
            smoke_report.tool_id
        ));
    }

    let mut rows = readiness
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    let expected_tool_ids = vec![EXPECTED_TOOL_ID.to_string()];
    let observed_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<Vec<_>>();
    if observed_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.recalibration readiness rows drifted: observed={observed_tool_ids:?} expected={expected_tool_ids:?}"
));
    }

    let report_rows = rows
        .iter()
        .map(|readiness_row| {
            build_bam_recalibration_complete_row(
                repo_root,
                &smoke_path,
                readiness_row,
                &smoke_report,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    let complete_row_count =
        report_rows.iter().filter(|row| row.coverage_status == "complete").count();
    let incomplete_row_count = report_rows.len().saturating_sub(complete_row_count);
    let violations = report_rows
        .iter()
        .filter(|row| row.coverage_status != "complete")
        .cloned()
        .collect::<Vec<_>>();

    Ok(project(ReportProjection {
        output_path,
        repo_root,
        smoke_report,
        expected_tool_ids,
        report_rows,
        complete_row_count,
        incomplete_row_count,
        violations,
    }))
}
