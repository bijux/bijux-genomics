use super::super::evidence::{ReadinessCommandPaths, SurfaceAssessment};
use super::fields::no_value_string;
use super::model::VcfPcaReadyRow;
use crate::pca::LocalVcfPcaSmokeReport;
use bijux_dna_bench_vcf_evidence::model::expectations::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH;
use bijux_dna_bench_vcf_evidence::model::parsers::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH;
use bijux_dna_bench_vcf_evidence::model::pca::PcaAdapterOutputProof;
use bijux_dna_bench_vcf_evidence::model::reports::DEFAULT_VCF_REPORT_MAP_PATH;
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
// The compatible flat wire record projects assessed surfaces and scientific smoke fields.
#[allow(
    clippy::too_many_lines,
    reason = "The flat persisted readiness row requires explicit field-for-field projection."
)]
pub(super) fn build_vcf_pca_ready_row(
    command_report: &ReadinessCommandPaths,
    binding: VcfStageReadinessBinding,
    output_proof: PcaAdapterOutputProof,
    smoke_report: Option<&LocalVcfPcaSmokeReport>,
) -> VcfPcaReadyRow {
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
    } = super::coverage::assess(&binding, &output_proof, smoke_report);
    VcfPcaReadyRow {
        result_id,
        stage_id: binding.retained_row.stage_id,
        tool_id: binding.retained_row.tool_id,
        tool_status: binding.retained_row.tool_status,
        corpus_id: binding.retained_row.corpus_id,
        asset_profile_id: binding.retained_row.asset_profile_id,
        adapter_id: binding.retained_row.adapter_id,
        parser_id: binding.retained_row.parser_id,
        schema_id: binding.retained_row.schema_id,
        retained_scope_state: binding.retained_row.scope_state,
        retained_scope_detail: binding.retained_row.scope_detail,
        retained_scope_proof_path: binding.retained_row.scope_proof_path,
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
        output_proof_path: output_proof.output_proof_path,
        raw_outputs: output_proof.raw_outputs,
        normalized_metrics_outputs: output_proof.normalized_metrics_outputs,
        manifest_output: output_proof.manifest_output,
        index_outputs: output_proof.index_outputs,
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
        smoke_input_vcf_path: smoke_report
            .map_or_else(no_value_string, |report| report.input_vcf_path.clone()),
        smoke_sample_metadata_path: smoke_report
            .map_or_else(no_value_string, |report| report.sample_metadata_path.clone()),
        smoke_population_metadata_path: smoke_report
            .map_or_else(no_value_string, |report| report.population_metadata_path.clone()),
        smoke_population_labels_manifest_path: smoke_report
            .map_or_else(no_value_string, |report| report.population_labels_manifest_path.clone()),
        smoke_pca_tsv_path: smoke_report
            .map_or_else(no_value_string, |report| report.pca_tsv_path.clone()),
        smoke_pca_json_path: smoke_report
            .map_or_else(no_value_string, |report| report.pca_json_path.clone()),
        smoke_source_eigenvec_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_eigenvec_path.clone()),
        smoke_source_eigenval_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_eigenval_path.clone()),
        smoke_source_pca_manifest_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_pca_manifest_path.clone()),
        smoke_source_logs_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_logs_path.clone()),
        smoke_stage_result_manifest_path: smoke_report
            .map_or_else(no_value_string, |report| report.stage_result_manifest_path.clone()),
        smoke_execution_mode: smoke_report
            .map_or_else(no_value_string, |report| report.execution_mode.clone()),
        smoke_tool_ok: smoke_report.is_some_and(|report| report.tool_ok),
        smoke_variant_count: smoke_report.map_or(0, |report| report.variant_count),
        smoke_sample_count: smoke_report.map_or(0, |report| report.sample_count),
        smoke_excluded_samples: smoke_report
            .map(|report| report.excluded_samples.clone())
            .unwrap_or_default(),
        smoke_unexpected_samples: smoke_report
            .map(|report| report.unexpected_samples.clone())
            .unwrap_or_default(),
        smoke_eigenvalues: smoke_report
            .map(|report| report.eigenvalues.clone())
            .unwrap_or_default(),
        smoke_rows: smoke_report.map(|report| report.rows.clone()).unwrap_or_default(),
        required_metric_names,
        missing_surfaces,
        coverage_status,
        reason,
    }
}
