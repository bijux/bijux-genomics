use super::{find_row, model::PublishedAdapterReport};
use anyhow::{bail, Result};
pub(crate) fn ensure_single_phasing_output(
    payload: &PublishedAdapterReport,
    tool_id: &str,
    expected_benchmark_status: &str,
    expected_total_rows: u64,
    expected_total_benchmark_ready_rows: u64,
    label: &str,
) -> Result<()> {
    if payload.row_count != Some(expected_total_rows)
        || payload.benchmark_ready_row_count != Some(expected_total_benchmark_ready_rows)
        || payload.parser_output_row_count != Some(expected_total_rows)
        || payload.indexed_row_count != Some(expected_total_rows)
        || payload.missing_input_test_passed_row_count != Some(expected_total_rows)
    {
        bail!(
            "{label} drifted: rows={}, benchmark_ready={}, parser_outputs={}, indexed={}, missing_input_passed={}",
            payload.row_count.unwrap_or_default(),
            payload.benchmark_ready_row_count.unwrap_or_default(),
            payload.parser_output_row_count.unwrap_or_default(),
            payload.indexed_row_count.unwrap_or_default(),
            payload.missing_input_test_passed_row_count.unwrap_or_default()
        );
    }
    let row = find_row(payload, tool_id, "vcf.phasing")?;
    if row.benchmark_status.as_deref() != Some(expected_benchmark_status)
        || row.argv_validation_passed != Some(true)
        || row.missing_input_test_passed != Some(true)
    {
        bail!(
            "{label} row drifted: tool=`{}`, stage=`{}`, benchmark_status=`{}`, argv_ok={}, missing_input_ok={}",
            row.tool_id.as_deref().unwrap_or(""),
            row.stage_id.as_deref().unwrap_or(""),
            row.benchmark_status.as_deref().unwrap_or(""),
            row.argv_validation_passed.unwrap_or(false),
            row.missing_input_test_passed.unwrap_or(false)
        );
    }
    Ok(())
}
