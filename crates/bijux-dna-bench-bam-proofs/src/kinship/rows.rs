use super::acquisition::{read_case_documents, CaseDocuments};
use super::checks::check_bam_locality_ready;
use super::checks::check_bam_pipeline_ready;
use super::checks::check_insufficient_case_report_ready;
use super::checks::check_insufficient_case_stage_metrics_ready;
use super::checks::check_insufficient_case_summary_ready;
use super::checks::check_local_smoke_ready;
use super::checks::check_parser_smoke_ready;
use super::checks::check_ready_case_pairwise_table_ready;
use super::checks::check_ready_case_report_ready;
use super::checks::check_ready_case_stage_metrics_ready;
use super::checks::check_ready_case_summary_ready;
use super::checks::check_vcf_isolation_ready;
use super::explanation::explain;
use super::explanation::SurfaceChecks;
use super::input::required_snapshot_string;
use super::input::required_snapshot_u64;
use super::models::BamKinshipCompleteRow;
use super::projection::project_row;
use super::projection::RowProjection;
use super::row_input::RowInputs;
use anyhow::anyhow;
use anyhow::Result;
pub(super) fn build_bam_kinship_complete_row(
    &RowInputs {
        repo_root,
        smoke_path,
        smoke_report,
        ready_case,
        insufficient_case,
        readiness_row,
        parser_row,
        bam_pipeline,
        bam_pipeline_node,
        vcf_pipeline,
    }: &RowInputs<'_>,
) -> Result<BamKinshipCompleteRow> {
    let CaseDocuments {
        ready_report,
        ready_summary,
        ready_stage_metrics,
        insufficient_report,
        insufficient_summary,
        insufficient_stage_metrics,
        ready_segments,
    } = read_case_documents(repo_root, ready_case, insufficient_case)?;
    let ready_pair = ready_case
        .pairwise_results
        .first()
        .ok_or_else(|| anyhow!("ready bam.kinship smoke case must keep one pairwise result"))?;

    let local_smoke_ready = check_local_smoke_ready(smoke_report, ready_case, insufficient_case);

    let ready_case_report_ready = check_ready_case_report_ready(&ready_report)?;
    let ready_case_summary_ready = check_ready_case_summary_ready(&ready_summary)?;
    let ready_case_pairwise_table_ready = check_ready_case_pairwise_table_ready(&ready_segments);
    let ready_case_stage_metrics_ready =
        check_ready_case_stage_metrics_ready(&ready_stage_metrics)?;

    let insufficient_case_report_ready =
        check_insufficient_case_report_ready(&insufficient_report)?;
    let insufficient_case_summary_ready =
        check_insufficient_case_summary_ready(&insufficient_summary)?;
    let insufficient_case_stage_metrics_ready =
        check_insufficient_case_stage_metrics_ready(&insufficient_stage_metrics)?;

    let parser_smoke_method = required_snapshot_string(parser_row, "method")?;
    let parser_smoke_status = required_snapshot_string(parser_row, "status")?;
    let parser_smoke_pair_count = required_snapshot_u64(parser_row, "pair_count")?;
    let parser_smoke_observed_max_overlap_snps =
        required_snapshot_u64(parser_row, "observed_max_overlap_snps")?;
    let parser_smoke_ready = check_parser_smoke_ready(
        parser_row,
        &parser_smoke_method,
        &parser_smoke_status,
        parser_smoke_pair_count,
        parser_smoke_observed_max_overlap_snps,
    );

    let bam_pipeline_ready = check_bam_pipeline_ready(bam_pipeline, bam_pipeline_node);
    let bam_locality_ready = check_bam_locality_ready(bam_pipeline);
    let vcf_isolation_ready = check_vcf_isolation_ready(vcf_pipeline);

    let (missing_surfaces, coverage_status, reason) = explain(
        readiness_row,
        SurfaceChecks {
            local_smoke_ready,
            ready_case_report_ready,
            ready_case_summary_ready,
            ready_case_pairwise_table_ready,
            ready_case_stage_metrics_ready,
            insufficient_case_report_ready,
            insufficient_case_summary_ready,
            insufficient_case_stage_metrics_ready,
            parser_smoke_ready,
            bam_pipeline_ready,
            bam_locality_ready,
            vcf_isolation_ready,
        },
    );
    Ok(project_row(RowProjection {
        repo_root,
        smoke_path,
        ready_case,
        insufficient_case,
        readiness_row,
        parser_row,
        bam_pipeline,
        bam_pipeline_node,
        vcf_pipeline,
        ready_pair,
        local_smoke_ready,
        ready_case_report_ready,
        ready_case_summary_ready,
        ready_case_pairwise_table_ready,
        ready_case_stage_metrics_ready,
        insufficient_case_report_ready,
        insufficient_case_summary_ready,
        insufficient_case_stage_metrics_ready,
        parser_smoke_method,
        parser_smoke_status,
        parser_smoke_pair_count,
        parser_smoke_observed_max_overlap_snps,
        parser_smoke_ready,
        bam_pipeline_ready,
        bam_locality_ready,
        vcf_isolation_ready,
        missing_surfaces,
        coverage_status,
        reason,
    }))
}
