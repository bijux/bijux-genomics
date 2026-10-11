use super::{find_row, model::PublishedAdapterReport};
use anyhow::{bail, Result};
pub(crate) fn ensure_imputation_family_output(
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
            "VCF imputation-family row `{stage_id}` / `{tool_id}` drifted: benchmark_status=`{}`, argv_ok={}, missing_input_ok={}",
            row.benchmark_status.as_deref().unwrap_or(""),
            row.argv_validation_passed.unwrap_or(false),
            row.missing_input_test_passed.unwrap_or(false)
        );
    }
    Ok(())
}
