use super::super::evidence::{ReadinessCommandPaths, SurfaceAssessment};
use super::fields::primary_executables_from_binding;
use super::model::VcfPrepareReferencePanelReadyRow;
use super::NO_VALUE;
use crate::reference_panel::LocalVcfPrepareReferencePanelSmokeReport;
use bijux_dna_bench_model::DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH;
use bijux_dna_bench_vcf_evidence::model::expectations::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH;
use bijux_dna_bench_vcf_evidence::model::outputs::DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH;
use bijux_dna_bench_vcf_evidence::model::parsers::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH;
use bijux_dna_bench_vcf_evidence::model::reports::DEFAULT_VCF_REPORT_MAP_PATH;
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
// The compatible flat wire record projects independently assessed surfaces and stage smoke fields.
pub(super) fn build_vcf_prepare_reference_panel_ready_row(
    command_report: &ReadinessCommandPaths,
    binding: &VcfStageReadinessBinding,
    smoke_report: Option<&LocalVcfPrepareReferencePanelSmokeReport>,
) -> VcfPrepareReferencePanelReadyRow {
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

    VcfPrepareReferencePanelReadyRow {
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
        all_domain_active_row_proof_path: if all_domain_active_row_present {
            DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH.to_string()
        } else {
            NO_VALUE.to_string()
        },
        command_ready,
        command_source: "vcf_bcftools_adapter".to_string(),
        command_step_count: binding.command_row.as_ref().map_or(0, |row| row.command_steps.len()),
        command_step_ids: binding
            .command_row
            .as_ref()
            .map(|row| row.command_steps.iter().map(|step| step.step_id.clone()).collect())
            .unwrap_or_default(),
        primary_executables: binding
            .command_row
            .as_ref()
            .map(primary_executables_from_binding)
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
            .map_or_else(|| NO_VALUE.to_string(), |row| row.section_id.clone()),
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
        smoke_command: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.command.clone()),
        smoke_output_root: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.output_root.clone()),
        smoke_input_vcf_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.input_vcf_path.clone()),
        smoke_raw_panel_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.raw_panel_path.clone()),
        smoke_panel_vcf_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.panel_vcf_path.clone()),
        smoke_panel_tbi_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.panel_tbi_path.clone()),
        smoke_panel_manifest_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.panel_manifest_path.clone()),
        smoke_overlap_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.overlap_path.clone()),
        smoke_panel_overlap_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.panel_overlap_path.clone()),
        smoke_panel_files_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.panel_files_path.clone()),
        smoke_overlap_tsv_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.overlap_tsv_path.clone()),
        smoke_chunks_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.chunks_path.clone()),
        smoke_metrics_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.metrics_path.clone()),
        smoke_stage_result_manifest_path: smoke_report.map_or_else(
            || NO_VALUE.to_string(),
            |report| report.stage_result_manifest_path.clone(),
        ),
        smoke_panel_id: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.panel_id.clone()),
        smoke_map_id: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.map_id.clone()),
        smoke_input_variants: smoke_report.map_or(0, |report| report.input_variants),
        smoke_output_variants: smoke_report.map_or(0, |report| report.output_variants),
        smoke_sample_count: smoke_report.map_or(0, |report| report.sample_count),
        smoke_sample_ids: smoke_report.map(|report| report.sample_ids.clone()).unwrap_or_default(),
        smoke_sample_consistent: smoke_report.is_some_and(|report| report.sample_consistent),
        smoke_duplicate_sites_removed: smoke_report
            .map_or(0, |report| report.duplicate_sites_removed),
        smoke_normalization_status: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.normalization_status.clone()),
        smoke_index_path: smoke_report
            .map_or_else(|| NO_VALUE.to_string(), |report| report.index_path.clone()),
        smoke_parseable: smoke_report.is_some_and(|report| report.parseable),
        smoke_gt_present: smoke_report.is_some_and(|report| report.gt_present),
        smoke_gl_present: smoke_report.is_some_and(|report| report.gl_present),
        smoke_validation_checks: smoke_report
            .map(|report| report.validation_checks.clone())
            .unwrap_or_default(),
        required_metric_names,
        missing_surfaces,
        coverage_status,
        reason,
    }
}
