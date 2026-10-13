use super::super::evidence::{ReadinessCommandPaths, SurfaceAssessment};
use super::fields::no_value_string;
use super::model::VcfCallDiploidReadyRow;
use crate::call_diploid::LocalVcfCallDiploidSmokeReport;
use bijux_dna_bench_vcf_evidence::{
    model::{
        expectations::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH,
        outputs::DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH,
        parsers::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH, reports::DEFAULT_VCF_REPORT_MAP_PATH,
    },
    snapshot::VcfStageReadinessBinding,
};
// The compatible flat wire record projects admitted surfaces and stage-specific smoke fields; coverage policy is evaluated separately.
#[allow(
    clippy::too_many_lines,
    reason = "The flat persisted readiness row requires explicit field-for-field projection."
)]
pub(super) fn build_vcf_call_diploid_ready_row(
    command_report: &ReadinessCommandPaths,
    binding: VcfStageReadinessBinding,
    smoke_report: Option<&LocalVcfCallDiploidSmokeReport>,
) -> VcfCallDiploidReadyRow {
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
    } = super::coverage::assess(&binding, smoke_report);
    VcfCallDiploidReadyRow {
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
        smoke_output_vcf_path: smoke_report
            .map_or_else(no_value_string, |report| report.output_vcf_path.clone()),
        smoke_output_tbi_path: smoke_report
            .map_or_else(no_value_string, |report| report.output_tbi_path.clone()),
        smoke_metrics_path: smoke_report
            .map_or_else(no_value_string, |report| report.metrics_path.clone()),
        smoke_stage_result_manifest_path: smoke_report
            .map_or_else(no_value_string, |report| report.stage_result_manifest_path.clone()),
        smoke_parseable: smoke_report.is_some_and(|report| report.parseable),
        smoke_diploid_compatible: smoke_report.is_some_and(|report| report.diploid_compatible),
        smoke_gt_present: smoke_report.is_some_and(|report| report.gt_present),
        smoke_gl_present: smoke_report.is_some_and(|report| report.gl_present),
        smoke_ploidy: smoke_report.map_or_else(no_value_string, |report| report.ploidy.to_string()),
        smoke_called_genotypes: smoke_report.map_or(0, |report| report.called_genotypes),
        smoke_heterozygous_count: smoke_report.map_or(0, |report| report.heterozygous_count),
        smoke_homozygous_ref_count: smoke_report.map_or(0, |report| report.homozygous_ref_count),
        smoke_homozygous_alt_count: smoke_report.map_or(0, |report| report.homozygous_alt_count),
        smoke_missing_count: smoke_report.map_or(0, |report| report.missing_count),
        smoke_sample_count: smoke_report.map_or(0, |report| report.sample_count),
        required_metric_names,
        missing_surfaces,
        coverage_status,
        reason,
    }
}
