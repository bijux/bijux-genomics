use super::declarations::EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_TOOL_IDS;
use super::models::BamContaminationCompleteReport;
use super::observations::LocalContaminationSmokeReport;
use super::report::project;
use super::report::ReportProjection;
use super::rows::build_bam_contamination_complete_row;
use crate::ports::NativeProofKind;
use crate::ProofContext;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
pub(super) fn build_bam_contamination_complete_report(
    context: &ProofContext<'_>,
    output_path: &Path,
) -> Result<BamContaminationCompleteReport> {
    let repo_root = context.repo_root;
    let readiness = context.readiness.contamination_sex_haplogroups(repo_root)?;
    let smoke_path = context.native.write(repo_root, NativeProofKind::Contamination)?;
    let smoke_report: LocalContaminationSmokeReport = crate::input::read_json(&smoke_path)?;

    if smoke_report.schema_version != EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected bam.contamination local-smoke schema `{}`",
            smoke_report.schema_version
        ));
    }
    if smoke_report.stage_id != EXPECTED_STAGE_ID {
        return Err(anyhow!(
            "unexpected bam.contamination local-smoke stage `{}`",
            smoke_report.stage_id
        ));
    }

    let mut smoke_tool_ids = smoke_report.tool_ids.clone();
    smoke_tool_ids.sort();
    let expected_tool_ids =
        EXPECTED_TOOL_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    if smoke_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.contamination local-smoke tool ids drifted: observed={smoke_tool_ids:?} expected={expected_tool_ids:?}"
));
    }

    let ready_cases = smoke_report
        .rows
        .iter()
        .filter(|row| row.proof_case == "ready")
        .map(|row| (row.tool_id.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();
    let insufficient_cases = smoke_report
        .rows
        .iter()
        .filter(|row| row.proof_case == "insufficient")
        .map(|row| (row.tool_id.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut rows = readiness
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    let observed_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<Vec<_>>();
    if observed_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.contamination readiness rows drifted: observed={observed_tool_ids:?} expected={expected_tool_ids:?}"
));
    }

    let mut report_rows = Vec::with_capacity(rows.len());
    for readiness_row in rows {
        let ready_case = ready_cases.get(&readiness_row.tool_id).ok_or_else(|| {
            anyhow!("missing bam.contamination ready smoke case for `{}`", readiness_row.tool_id)
        })?;
        let insufficient_case =
            insufficient_cases.get(&readiness_row.tool_id).ok_or_else(|| {
                anyhow!(
                    "missing bam.contamination insufficient smoke case for `{}`",
                    readiness_row.tool_id
                )
            })?;
        report_rows.push(build_bam_contamination_complete_row(
            repo_root,
            &smoke_path,
            &smoke_report,
            &readiness_row,
            ready_case,
            insufficient_case,
        )?);
    }

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
