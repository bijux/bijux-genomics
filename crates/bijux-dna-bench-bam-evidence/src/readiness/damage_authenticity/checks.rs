use super::declarations::EXPECTED_FIXTURE_ID;
use super::declarations::EXPECTED_SCHEMA_REQUIRED_KEYS;
use super::projection::RowInput;
use crate::command_coverage::BamAdapterCoverageKind;
use crate::command_coverage::BamBenchmarkStatus;
use crate::command_coverage::BamReadinessGapKind;
use crate::output_contract::BamAdapterOutputContractStatus;
#[derive(Clone, Copy)]
// Each flag records a separate evidence surface; combinations retain diagnostic meaning.
#[allow(clippy::struct_excessive_bools)]
pub(super) struct SurfaceChecks {
    pub active_scope_ready: bool,
    pub command_ready: bool,
    pub output_ready: bool,
    pub parser_ready: bool,
    pub expected_result_ready: bool,
    pub report_ready: bool,
    pub schema_ready: bool,
    pub local_smoke_ready: bool,
}
pub(super) fn evaluate(input: &RowInput<'_>) -> SurfaceChecks {
    let RowInput {
        active_row,
        command_row,
        output_row,
        parser_row,
        expected_row,
        report_map_row,
        schema_contract,
        local_smoke,
        spec,
        ..
    } = *input;
    let active_scope_ready = active_row.support_status == "supported"
        && active_row.adapter_status == "runnable"
        && active_row.parser_status == "parser_fixture_validated"
        && active_row.corpus_status.starts_with("fixture:");
    let command_ready = command_row.benchmark_status == BamBenchmarkStatus::BenchmarkReady
        && command_row.adapter_coverage == BamAdapterCoverageKind::Covered
        && command_row.readiness_gap == BamReadinessGapKind::None;
    let output_ready = output_row.output_contract_status
        == BamAdapterOutputContractStatus::Complete
        && output_row.normalized_metrics_output_id.as_deref()
            == Some(spec.normalized_metrics_output_id)
        && spec.required_output_ids.iter().all(|output_id| {
            output_row.stage_output_ids.iter().any(|candidate| candidate == output_id)
        })
        && output_row.stdout_path_template.is_some()
        && output_row.stderr_path_template.is_some()
        && output_row.stage_result_manifest_path_template.is_some();
    let parser_ready = parser_row.coverage_status
        == crate::parser_coverage::BamParserFixtureCoverageStatus::Covered;
    let expected_result_ready = expected_row.fixture_id == EXPECTED_FIXTURE_ID
        && expected_row.normalized_metrics_output_id.as_deref()
            == Some(spec.normalized_metrics_output_id)
        && !expected_row.result_root.is_empty()
        && !expected_row.stage_result_manifest_path.is_empty()
        && !expected_row.stdout_path.is_empty()
        && !expected_row.stderr_path.is_empty();
    let report_ready = report_map_row.report_section_id == spec.report_section_id
        && report_map_row.summary_table_id == spec.summary_table_id
        && report_map_row.anchor_tool_id == spec.report_anchor_tool_id
        && report_map_row.anchor_support_status == "supported";
    let schema_ready = schema_contract.extension_id == spec.schema_extension_id
        && EXPECTED_SCHEMA_REQUIRED_KEYS
            .iter()
            .all(|field| schema_contract.required_keys.iter().any(|candidate| candidate == field));
    let local_smoke_ready = spec
        .required_local_smoke_fields
        .iter()
        .all(|field| local_smoke.observed_fields.iter().any(|candidate| candidate == field))
        && !local_smoke.artifact_paths.is_empty();

    SurfaceChecks {
        active_scope_ready,
        command_ready,
        output_ready,
        parser_ready,
        expected_result_ready,
        report_ready,
        schema_ready,
        local_smoke_ready,
    }
}
