use super::super::evidence::{ReadinessCommandPaths, SurfaceAssessment};
use super::model::VcfAdmixtureReadyRow;
use super::NO_VALUE;
use crate::admixture::LocalVcfAdmixtureSmokeReport;
use bijux_dna_bench_vcf_evidence::model::expectations::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH;
use bijux_dna_bench_vcf_evidence::model::outputs::DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH;
use bijux_dna_bench_vcf_evidence::model::parsers::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH;
use bijux_dna_bench_vcf_evidence::model::reports::DEFAULT_VCF_REPORT_MAP_PATH;
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
use std::collections::BTreeSet;
// The compatible flat wire record projects independently assessed surfaces and stage smoke fields.
pub(super) fn build_vcf_admixture_ready_row(
    command_report: &ReadinessCommandPaths,
    binding: &VcfStageReadinessBinding,
    smoke_report: Option<&LocalVcfAdmixtureSmokeReport>,
) -> VcfAdmixtureReadyRow {
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
    let primary_executables = binding
        .command_row
        .as_ref()
        .map(|row| {
            row.command_steps
                .iter()
                .filter_map(|step| step.argv.first().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    VcfAdmixtureReadyRow {
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
        all_domain_active_row_proof_path: binding.active_row.as_ref().map_or_else(
            || NO_VALUE.to_string(),
            |_| {
                "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv"
                    .to_string()
            },
        ),
        command_ready,
        command_source: binding
            .command_row
            .as_ref()
            .map_or_else(|| NO_VALUE.to_string(), |row| row.command_source.clone()),
        command_step_count: binding.command_row.as_ref().map_or(0, |row| row.command_steps.len()),
        command_step_ids: binding
            .command_row
            .as_ref()
            .map(|row| row.command_steps.iter().map(|step| step.step_id.clone()).collect())
            .unwrap_or_default(),
        primary_executables,
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
            .map_or_else(|| NO_VALUE.to_string(), |row| row.manifest.clone()),
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
            .map_or_else(|| NO_VALUE.to_string(), |row| row.parser_fixture_parser_id.clone()),
        parser_fixture_schema_id: binding
            .parser_row
            .as_ref()
            .map_or_else(|| NO_VALUE.to_string(), |row| row.parser_fixture_schema_id.clone()),
        parser_fixture_path: binding
            .parser_row
            .as_ref()
            .map_or_else(|| NO_VALUE.to_string(), |row| row.parser_fixture_root_path.clone()),
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
            .report_row
            .as_ref()
            .map(|row| row.section_id.clone())
            .or_else(|| binding.expected_row.as_ref().map(|row| row.report_section.clone()))
            .unwrap_or_else(|| NO_VALUE.to_string()),
        report_ready,
        report_map_proof_path: DEFAULT_VCF_REPORT_MAP_PATH.to_string(),
        summary_table_id: binding
            .report_row
            .as_ref()
            .map_or_else(|| NO_VALUE.to_string(), |row| row.summary_table.clone()),
        report_metric_columns: binding
            .report_row
            .as_ref()
            .map(|row| row.metric_columns.clone())
            .unwrap_or_default(),
        smoke_ready,
        smoke_tool_id: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.tool_id.clone()),
        smoke_command: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.command.clone()),
        smoke_output_root: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.output_root.clone()),
        smoke_input_vcf_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.input_vcf_path.clone()),
        smoke_sample_metadata_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.sample_metadata_path.clone()),
        smoke_population_metadata_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.population_metadata_path.clone()),
        smoke_population_labels_manifest_path: smoke_report.map_or_else(
            || NO_VALUE.to_string(),
            |report| report.population_labels_manifest_path.clone(),
        ),
        smoke_admixture_tsv_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.admixture_tsv_path.clone()),
        smoke_admixture_json_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.admixture_json_path.clone()),
        smoke_source_q_matrix_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.source_q_matrix_path.clone()),
        smoke_source_k_selection_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.source_k_selection_path.clone()),
        smoke_source_logs_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.source_logs_path.clone()),
        smoke_stage_result_manifest_path: smoke_report.map_or_else(
            || NO_VALUE.to_string(),
            |report| report.stage_result_manifest_path.clone(),
        ),
        smoke_execution_mode: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.execution_mode.clone()),
        smoke_tool_ok: smoke_report.is_some_and(|report| report.tool_ok),
        smoke_selected_k: smoke_report.map_or(0, |report| report.selected_k),
        smoke_status: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.status.clone()),
        smoke_sample_count: smoke_report.map_or(0, |report| report.sample_count),
        smoke_population_count: smoke_report.map_or(0, |report| report.population_count),
        smoke_cluster_headers: smoke_report
            .map(|report| report.cluster_headers.clone())
            .unwrap_or_default(),
        smoke_rows: smoke_report.map(|report| report.rows.clone()).unwrap_or_default(),
        required_metric_names,
        missing_surfaces,
        coverage_status,
        reason,
    }
}
