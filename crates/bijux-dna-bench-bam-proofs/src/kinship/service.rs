use super::contracts::find_case;
use super::contracts::find_parser_smoke_row;
use super::contracts::find_pipeline_node;
use super::declarations::EXPECTED_BAM_PIPELINE_ID;
use super::declarations::EXPECTED_INSUFFICIENT_CASE_SAMPLE_ID;
use super::declarations::EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION;
use super::declarations::EXPECTED_READY_CASE_SAMPLE_ID;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_TOOL_IDS;
use super::declarations::EXPECTED_VCF_PIPELINE_ID;
use super::models::BamKinshipCompleteReport;
use super::observations::LocalKinshipSmokeReport;
use super::report::project_report;
use super::row_input::RowInputs;
use super::rows::build_bam_kinship_complete_row;
use crate::downstream::observations::ParserObservations;
use crate::downstream::DownstreamContext;
use crate::ports::NativeProofKind;
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
pub(super) fn build_bam_kinship_complete_report(
    context: &DownstreamContext<'_>,
    output_path: &Path,
) -> Result<BamKinshipCompleteReport> {
    let repo_root = context.proofs.repo_root;
    let readiness = context.kinship.kinship(repo_root)?;
    let smoke_path = context.proofs.native.write(repo_root, NativeProofKind::Kinship)?;
    let smoke_report: LocalKinshipSmokeReport = crate::input::read_json(&smoke_path)?;
    let parser_smoke = ParserObservations { rows: context.parsers.all(repo_root)? };
    let bam_pipeline_report_path = repo_root
        .join("artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-kinship.json");
    let bam_pipeline = context.pipelines.validate(
        repo_root,
        EXPECTED_BAM_PIPELINE_ID,
        &bam_pipeline_report_path,
    )?;
    let vcf_pipeline_report_path = repo_root.join(
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-genotyping-to-vcf-downstream.json",
    );
    let vcf_pipeline = context.pipelines.validate(
        repo_root,
        EXPECTED_VCF_PIPELINE_ID,
        &vcf_pipeline_report_path,
    )?;

    if smoke_report.schema_version != EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected bam.kinship local-smoke schema `{}`",
            smoke_report.schema_version
        ));
    }
    if smoke_report.stage_id != EXPECTED_STAGE_ID {
        return Err(anyhow!(
            "unexpected bam.kinship local-smoke stage `{}`",
            smoke_report.stage_id
        ));
    }
    if smoke_report.case_count != 2 || smoke_report.cases.len() != 2 {
        return Err(anyhow!("bam.kinship local-smoke report must keep exactly two governed cases"));
    }

    let ready_case = find_case(&smoke_report, EXPECTED_READY_CASE_SAMPLE_ID)?;
    let insufficient_case = find_case(&smoke_report, EXPECTED_INSUFFICIENT_CASE_SAMPLE_ID)?;
    let parser_row = find_parser_smoke_row(&parser_smoke)?;
    let bam_pipeline_node = find_pipeline_node(&bam_pipeline, EXPECTED_STAGE_ID)?;

    let mut rows = readiness
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    let expected_tool_ids =
        EXPECTED_TOOL_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let observed_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<Vec<_>>();
    if observed_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.kinship readiness rows drifted: observed={observed_tool_ids:?} expected={expected_tool_ids:?}"
        ));
    }

    let report_rows = rows
        .iter()
        .map(|readiness_row| {
            build_bam_kinship_complete_row(&RowInputs {
                repo_root,
                smoke_path: &smoke_path,
                smoke_report: &smoke_report,
                ready_case,
                insufficient_case,
                readiness_row,
                parser_row,
                bam_pipeline: &bam_pipeline,
                bam_pipeline_node,
                vcf_pipeline: &vcf_pipeline,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(project_report(
        repo_root,
        output_path,
        report_rows,
        expected_tool_ids,
        smoke_report.case_count,
    ))
}
