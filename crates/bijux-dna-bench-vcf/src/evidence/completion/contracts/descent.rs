use super::{find_row, model::PublishedAdapterReport};
use anyhow::{bail, Result};
pub(crate) fn ensure_descent_family_output_contract(
    payload: &PublishedAdapterReport,
) -> Result<()> {
    if payload.row_count != Some(5)
        || payload.tool_count != Some(5)
        || payload.benchmark_ready_row_count != Some(5)
        || payload.parser_output_row_count != Some(5)
        || payload.normalized_output_row_count != Some(5)
        || payload.missing_input_test_passed_row_count != Some(5)
    {
        bail!(
            "VCF descent-family coverage drifted: rows={}, tools={}, benchmark_ready={}, parser_outputs={}, normalized_outputs={}, missing_input_passed={}",
            payload.row_count.unwrap_or_default(),
            payload.tool_count.unwrap_or_default(),
            payload.benchmark_ready_row_count.unwrap_or_default(),
            payload.parser_output_row_count.unwrap_or_default(),
            payload.normalized_output_row_count.unwrap_or_default(),
            payload.missing_input_test_passed_row_count.unwrap_or_default()
        );
    }
    Ok(())
}
pub(crate) fn ensure_descent_family_output(
    payload: &PublishedAdapterReport,
    tool_id: &str,
    stage_id: &str,
    expected_benchmark_status: &str,
) -> Result<()> {
    let row = find_row(payload, tool_id, stage_id)?;
    if row.benchmark_status.as_deref() != Some(expected_benchmark_status)
        || row.argv_validation_passed != Some(true)
        || row.missing_input_test_passed != Some(true)
    {
        bail!(
            "VCF descent-family row `{stage_id}` / `{tool_id}` drifted: benchmark_status=`{}`, argv_ok={}, missing_input_ok={}",
            row.benchmark_status.as_deref().unwrap_or(""),
            row.argv_validation_passed.unwrap_or(false),
            row.missing_input_test_passed.unwrap_or(false)
        );
    }
    Ok(())
}
