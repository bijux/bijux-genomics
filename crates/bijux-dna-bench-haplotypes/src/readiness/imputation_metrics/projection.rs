use super::super::evidence::{ReadinessCommandPaths, SurfaceAssessment};
use super::fields::{no_value_string, round_metric};
use super::model::VcfImputationMetricsReadyRow;
use crate::imputation_metrics::LocalVcfImputationMetricsSmokeReport;
use bijux_dna_bench_vcf_evidence::model::expectations::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH;
use bijux_dna_bench_vcf_evidence::model::outputs::DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH;
use bijux_dna_bench_vcf_evidence::model::parsers::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH;
use bijux_dna_bench_vcf_evidence::model::reports::DEFAULT_VCF_REPORT_MAP_PATH;
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
// The compatible flat wire record projects independently assessed surfaces and stage smoke fields.
pub(super) fn build_vcf_imputation_metrics_ready_row(
    command_report: &ReadinessCommandPaths,
    binding: &VcfStageReadinessBinding,
    smoke_report: Option<&LocalVcfImputationMetricsSmokeReport>,
) -> VcfImputationMetricsReadyRow {
    let SurfaceAssessment {
        result_id,
        required_metric_names,
        missing_surfaces,
        all_domain_active_row_present,
        command_ready,
        output_ready,
        parser_ready,
        expected_result_ready,
        report_ready,
        smoke_ready,
        coverage_status,
        reason,
    } = super::coverage::assess(binding, smoke_report);

    VcfImputationMetricsReadyRow {
        result_id,
        stage_id: binding.retained_row.stage_id.clone(),
        tool_id: binding.retained_row.tool_id.clone(),
        tool_status: binding.retained_row.tool_status.clone(),
        corpus_id: binding.retained_row.corpus_id.clone(),
        asset_profile_id: binding.retained_row.asset_profile_id.clone(),
        adapter_id: binding.retained_row.adapter_id.clone(),
        parser_id: binding.retained_row.parser_id.clone(),
        schema_id: binding.retained_row.schema_id.clone(),
        retained_scope_state: binding.retained_row.scope_state.clone(),
        retained_scope_detail: binding.retained_row.scope_detail.clone(),
        retained_scope_proof_path: binding.retained_row.scope_proof_path.clone(),
        all_domain_active_row_present,
        all_domain_active_row_proof_path:
            "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv"
                .to_string(),
        command_ready,
        command_source: binding
            .command_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.command_source.clone()),
        command_step_count: binding.command_row.as_ref().map_or(0, |row| row.command_steps.len()),
        command_step_ids: binding
            .command_row
            .as_ref()
            .map(|row| row.command_steps.iter().map(|step| step.step_id.clone()).collect())
            .unwrap_or_default(),
        primary_executables: binding
            .command_row
            .as_ref()
            .map(|row| {
                row.command_steps
                    .iter()
                    .map(|step| step.argv.first().cloned().unwrap_or_else(no_value_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        command_output_path: command_report.output_path.clone(),
        command_argv_output_path: command_report.argv_output_path.clone(),
        output_ready,
        output_proof_path: DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH.to_string(),
        raw_outputs: binding
            .output_row
            .as_ref()
            .map(|row| row.raw_outputs.clone())
            .unwrap_or_default(),
        normalized_metrics_outputs: binding
            .output_row
            .as_ref()
            .map(|row| row.normalized_metrics.clone())
            .unwrap_or_default(),
        manifest_output: binding
            .output_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.manifest.clone()),
        index_outputs: binding
            .output_row
            .as_ref()
            .map(|row| row.index_outputs.clone())
            .unwrap_or_default(),
        parser_ready,
        parser_proof_path: DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH.to_string(),
        parser_fixture_parser_id: binding
            .parser_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.parser_fixture_parser_id.clone()),
        parser_fixture_schema_id: binding
            .parser_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.parser_fixture_schema_id.clone()),
        parser_fixture_path: binding
            .parser_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.parser_fixture_root_path.clone()),
        expected_result_ready,
        expected_result_proof_path: DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH.to_string(),
        expected_outputs: binding
            .expected_row
            .as_ref()
            .map(|row| row.expected_outputs.clone())
            .unwrap_or_default(),
        expected_metrics: binding
            .expected_row
            .as_ref()
            .map(|row| row.expected_metrics.clone())
            .unwrap_or_default(),
        report_section_id: binding
            .expected_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.report_section.clone()),
        report_ready,
        report_map_proof_path: DEFAULT_VCF_REPORT_MAP_PATH.to_string(),
        summary_table_id: binding
            .report_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.summary_table.clone()),
        report_metric_columns: binding
            .report_row
            .as_ref()
            .map(|row| row.metric_columns.clone())
            .unwrap_or_default(),
        smoke_ready,
        smoke_command: smoke_report.map_or_else(no_value_string, |report| report.command.clone()),
        smoke_output_root: smoke_report
            .map_or_else(no_value_string, |report| report.output_root.clone()),
        smoke_imputation_metrics_path: smoke_report
            .map_or_else(no_value_string, |report| report.imputation_metrics_path.clone()),
        smoke_source_imputation_qc_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_imputation_qc_path.clone()),
        smoke_source_impute_smoke_metrics_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_impute_smoke_metrics_path.clone()),
        smoke_source_imputation_manifest_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_imputation_manifest_path.clone()),
        smoke_stage_result_manifest_path: smoke_report
            .map_or_else(no_value_string, |report| report.stage_result_manifest_path.clone()),
        smoke_status: smoke_report.map_or_else(no_value_string, |report| report.status.clone()),
        smoke_concordance: smoke_report.and_then(|report| report.concordance.map(round_metric)),
        smoke_mean_info_score: smoke_report
            .map_or(0.0, |report| round_metric(report.mean_info_score)),
        smoke_r2_available: smoke_report.is_some_and(|report| report.r2_available),
        smoke_dosage_r2: smoke_report.and_then(|report| report.dosage_r2.map(round_metric)),
        smoke_low_confidence_sites: smoke_report.map_or(0, |report| report.low_confidence_sites),
        smoke_masked_truth_sites: smoke_report.map_or(0, |report| report.masked_truth_sites),
        smoke_missing_quality_fields: smoke_report
            .map(|report| report.missing_quality_fields.clone())
            .unwrap_or_default(),
        required_metric_names,
        missing_surfaces,
        coverage_status,
        reason,
    }
}
