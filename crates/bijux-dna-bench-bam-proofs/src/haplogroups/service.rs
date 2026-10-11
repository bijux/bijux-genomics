use super::declarations::EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION;
use super::declarations::EXPECTED_REFERENCE_BUILD;
use super::declarations::EXPECTED_REFERENCE_PANEL_ID;
use super::declarations::EXPECTED_SAMPLE_ID;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_TOOL_IDS;
use super::models::BamHaplogroupsCompleteReport;
use super::observations::LocalHaplogroupsSmokeReport;
use super::report::project;
use super::report::ReportProjection;
use super::rows::build_bam_haplogroups_complete_row;
use crate::ports::NativeProofKind;
use crate::ProofContext;
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
pub(super) fn build_bam_haplogroups_complete_report(
    context: &ProofContext<'_>,
    output_path: &Path,
) -> Result<BamHaplogroupsCompleteReport> {
    let repo_root = context.repo_root;
    let readiness = context.readiness.contamination_sex_haplogroups(repo_root)?;
    let smoke_path = context.native.write(repo_root, NativeProofKind::Haplogroups)?;
    let smoke_report: LocalHaplogroupsSmokeReport = crate::input::read_json(&smoke_path)?;

    if smoke_report.schema_version != EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke schema `{}`",
            smoke_report.schema_version
        ));
    }
    if smoke_report.stage_id != EXPECTED_STAGE_ID {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke stage `{}`",
            smoke_report.stage_id
        ));
    }
    if smoke_report.tool_id != EXPECTED_TOOL_IDS[0] {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke tool `{}`",
            smoke_report.tool_id
        ));
    }
    if smoke_report.sample_id != EXPECTED_SAMPLE_ID {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke sample `{}`",
            smoke_report.sample_id
        ));
    }
    if smoke_report.reference_panel_id != EXPECTED_REFERENCE_PANEL_ID {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke reference_panel_id `{}`",
            smoke_report.reference_panel_id
        ));
    }
    if smoke_report.reference_build != EXPECTED_REFERENCE_BUILD {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke reference_build `{}`",
            smoke_report.reference_build
        ));
    }

    let ready_case = smoke_report
        .rows
        .iter()
        .find(|row| row.proof_case == "ready")
        .cloned()
        .ok_or_else(|| anyhow!("missing bam.haplogroups ready case"))?;
    let insufficient_case = smoke_report
        .rows
        .iter()
        .find(|row| row.proof_case == "insufficient")
        .cloned()
        .ok_or_else(|| anyhow!("missing bam.haplogroups insufficient case"))?;

    let expected_tool_ids =
        EXPECTED_TOOL_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let mut rows = readiness
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    let observed_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<Vec<_>>();
    if observed_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.haplogroups readiness rows drifted: observed={observed_tool_ids:?} expected={expected_tool_ids:?}"
));
    }

    let report_rows = rows
        .iter()
        .map(|readiness_row| {
            build_bam_haplogroups_complete_row(
                repo_root,
                &smoke_path,
                &smoke_report,
                readiness_row,
                &ready_case,
                &insufficient_case,
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
