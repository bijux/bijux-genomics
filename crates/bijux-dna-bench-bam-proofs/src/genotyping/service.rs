use super::contracts::find_pipeline_node;
use super::contracts::find_vcf_qc_row;
use super::contracts::find_vcf_stats_row;
use super::declarations::EXPECTED_BAM_PIPELINE_ID;
use super::declarations::EXPECTED_CROSS_PIPELINE_ID;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_TOOL_ID;
use super::models::BamGenotypingCompleteReport;
use super::observations::LocalGenotypingPlan;
use super::report::project_report;
use super::row_input::RowInputs;
use super::rows::build_bam_genotyping_complete_row;
use crate::downstream::DownstreamContext;
use crate::ports::NativeProofKind;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_core::id_catalog::VCF_FILTER;
use std::path::Path;
pub(super) fn build_bam_genotyping_complete_report(
    context: &DownstreamContext<'_>,
    output_path: &Path,
) -> Result<BamGenotypingCompleteReport> {
    let repo_root = context.proofs.repo_root;
    let readiness = context.proofs.readiness.recalibration_genotyping(repo_root)?;
    let local_plan_path = context.proofs.native.write(repo_root, NativeProofKind::Genotyping)?;
    let local_plan: LocalGenotypingPlan = crate::input::read_json(&local_plan_path)?;
    let parser_row = context.parsers.genotyping(repo_root)?;
    let vcf_stats = context.variants.counts(repo_root)?;
    let vcf_qc = context.variants.missingness(repo_root)?;
    let bam_pipeline_report_path = repo_root
        .join("artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-genotyping.json");
    let bam_pipeline = context.pipelines.validate(
        repo_root,
        EXPECTED_BAM_PIPELINE_ID,
        &bam_pipeline_report_path,
    )?;
    let cross_pipeline_report_path = repo_root.join(
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-genotyping-to-vcf-downstream.json",
    );
    let cross_pipeline = context.pipelines.validate(
        repo_root,
        EXPECTED_CROSS_PIPELINE_ID,
        &cross_pipeline_report_path,
    )?;

    if local_plan.stage_id != EXPECTED_STAGE_ID {
        return Err(anyhow!(
            "unexpected bam.genotyping local-plan stage `{}`",
            local_plan.stage_id
        ));
    }
    if local_plan.tool_id != EXPECTED_TOOL_ID {
        return Err(anyhow!("unexpected bam.genotyping local-plan tool `{}`", local_plan.tool_id));
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
            "bam.genotyping readiness rows drifted: observed={observed_tool_ids:?} expected={expected_tool_ids:?}"
        ));
    }

    let stats_row = find_vcf_stats_row(&vcf_stats)?;
    let qc_row = find_vcf_qc_row(&vcf_qc)?;
    let bam_pipeline_node = find_pipeline_node(&bam_pipeline, EXPECTED_STAGE_ID)?;
    let cross_pipeline_genotyping_node = find_pipeline_node(&cross_pipeline, EXPECTED_STAGE_ID)?;
    let cross_pipeline_vcf_filter_node = find_pipeline_node(&cross_pipeline, VCF_FILTER)?;

    let report_rows = rows
        .iter()
        .map(|readiness_row| {
            build_bam_genotyping_complete_row(&RowInputs {
                repo_root,
                local_plan_path: &local_plan_path,
                readiness_row,
                local_plan: &local_plan,
                parser_row: &parser_row,
                bam_pipeline: &bam_pipeline,
                bam_pipeline_node,
                cross_pipeline: &cross_pipeline,
                cross_pipeline_genotyping_node,
                cross_pipeline_vcf_filter_node,
                vcf_stats: &vcf_stats,
                stats_row,
                vcf_qc: &vcf_qc,
                qc_row,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(project_report(repo_root, output_path, report_rows, expected_tool_ids))
}
