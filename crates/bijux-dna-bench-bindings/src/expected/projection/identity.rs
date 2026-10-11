use crate::ExpectedContext;
pub(super) fn result_row_id(
    domain: &str,
    fixture_id: &str,
    stage_id: &str,
    sample_scope: &str,
    tool_id: &str,
) -> String {
    bijux_dna_core::ids::build_sample_scoped_benchmark_result_id(
        domain,
        fixture_id,
        stage_id,
        sample_scope,
        tool_id,
    )
}
pub(super) fn result_root_path(
    context: &ExpectedContext<'_>,
    fixture_id: &str,
    stage_id: &str,
    sample_scope: &str,
    tool_id: &str,
) -> String {
    let result_root = context.result_root;
    let run_id = context.run_id;
    format!("{result_root}/runs/{run_id}/{fixture_id}/{stage_id}/{sample_scope}/{tool_id}")
}
