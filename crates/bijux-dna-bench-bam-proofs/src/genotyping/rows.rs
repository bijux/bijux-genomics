use super::checks::check_bam_pipeline_ready;
use super::checks::check_cross_pipeline_ready;
use super::checks::check_downstream_handoff_ready;
use super::checks::check_downstream_missingness_ready;
use super::checks::check_downstream_variant_metrics_ready;
use super::checks::check_local_plan_ready;
use super::checks::check_parser_smoke_ready;
use super::checks::check_producer_contract_ready;
use super::contracts::find_plan_output;
use super::explanation::explain;
use super::explanation::SurfaceChecks;
use super::models::BamGenotypingCompleteRow;
use super::parser::{observe_parser, ParserSnapshot};
use super::projection::project_row;
use super::projection::RowProjection;
use super::row_input::RowInputs;
use anyhow::anyhow;
use anyhow::Result;
pub(super) fn build_bam_genotyping_complete_row(
    &RowInputs {
        repo_root,
        local_plan_path,
        readiness_row,
        local_plan,
        parser_row,
        bam_pipeline,
        bam_pipeline_node,
        cross_pipeline,
        cross_pipeline_genotyping_node,
        cross_pipeline_vcf_filter_node,
        vcf_stats,
        stats_row,
        vcf_qc,
        qc_row,
    }: &RowInputs<'_>,
) -> Result<BamGenotypingCompleteRow> {
    let summary_path = find_plan_output(&local_plan.io.outputs, "summary")?;
    let stage_metrics_path = find_plan_output(&local_plan.io.outputs, "stage_metrics")?;
    let genotyping_bcf_path = find_plan_output(&local_plan.io.outputs, "genotyping_bcf")?;
    let genotyping_vcf_path = find_plan_output(&local_plan.io.outputs, "genotyping_vcf")?;
    let genotyping_vcf_tbi_path = find_plan_output(&local_plan.io.outputs, "genotyping_vcf_tbi")?;
    let genotyping_gl_path = find_plan_output(&local_plan.io.outputs, "genotyping_gl")?;
    let command_shell = local_plan
        .command
        .template
        .last()
        .cloned()
        .ok_or_else(|| anyhow!("bam.genotyping local plan must carry a shell command"))?;

    let local_plan_ready = check_local_plan_ready(
        local_plan,
        &genotyping_bcf_path,
        &genotyping_vcf_path,
        &command_shell,
    );

    let producer_contract_ready = check_producer_contract_ready(
        local_plan,
        &genotyping_bcf_path,
        &genotyping_vcf_path,
        &genotyping_vcf_tbi_path,
        &genotyping_gl_path,
    );

    let ParserSnapshot {
        call_rate: parser_smoke_call_rate,
        mean_posterior: parser_smoke_mean_posterior,
        reference_fasta: parser_smoke_reference_fasta,
        sites_vcf: parser_smoke_sites_vcf,
    } = observe_parser(parser_row);
    let parser_smoke_ready = check_parser_smoke_ready(
        parser_row,
        parser_smoke_call_rate,
        parser_smoke_mean_posterior,
        &parser_smoke_reference_fasta,
        &parser_smoke_sites_vcf,
    );

    let bam_pipeline_ready = check_bam_pipeline_ready(bam_pipeline, bam_pipeline_node);
    let cross_pipeline_ready =
        check_cross_pipeline_ready(cross_pipeline, cross_pipeline_genotyping_node);
    let downstream_handoff_ready = check_downstream_handoff_ready(cross_pipeline_vcf_filter_node);

    let downstream_variant_metrics_ready = check_downstream_variant_metrics_ready(stats_row);
    let downstream_missingness_ready = check_downstream_missingness_ready(qc_row);

    let (missing_surfaces, coverage_status, reason) = explain(
        readiness_row,
        SurfaceChecks {
            local_plan_ready,
            producer_contract_ready,
            parser_smoke_ready,
            bam_pipeline_ready,
            cross_pipeline_ready,
            downstream_handoff_ready,
            downstream_variant_metrics_ready,
            downstream_missingness_ready,
        },
    );
    Ok(project_row(RowProjection {
        repo_root,
        local_plan_path,
        readiness_row,
        local_plan,
        parser_row,
        bam_pipeline,
        bam_pipeline_node,
        cross_pipeline,
        cross_pipeline_genotyping_node,
        cross_pipeline_vcf_filter_node,
        vcf_stats,
        stats_row,
        vcf_qc,
        qc_row,
        summary_path,
        stage_metrics_path,
        genotyping_bcf_path,
        genotyping_vcf_path,
        genotyping_vcf_tbi_path,
        genotyping_gl_path,
        local_plan_ready,
        producer_contract_ready,
        parser_smoke_call_rate,
        parser_smoke_mean_posterior,
        parser_smoke_reference_fasta,
        parser_smoke_sites_vcf,
        parser_smoke_ready,
        bam_pipeline_ready,
        cross_pipeline_ready,
        downstream_handoff_ready,
        downstream_variant_metrics_ready,
        downstream_missingness_ready,
        missing_surfaces,
        coverage_status,
        reason,
    }))
}
