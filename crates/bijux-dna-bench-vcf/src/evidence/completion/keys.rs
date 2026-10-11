use super::model::VcfBindingKey;
pub(super) fn binding_key_from_matrix_row(
    row: &bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow,
) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}
pub(super) fn binding_key_from_local_smoke_row(
    row: &crate::evidence::smoke::VcfLocalContainerSmokeRow,
) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}
pub(super) fn binding_key_from_parser_fixture_row(
    row: &bijux_dna_bench_vcf_evidence::model::parsers::VcfParserFixtureCoverageRow,
) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}
pub(super) fn binding_key_from_rendered_command_row(
    row: &bijux_dna_bench_vcf_evidence::model::commands::VcfRenderedCommandRow,
) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}
pub(super) fn binding_key_from_expected_result_row(
    row: &bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow,
) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}
pub(super) fn binding_key_from_report_map_row(
    row: &bijux_dna_bench_vcf_evidence::model::reports::VcfReportMapRow,
) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}
