use anyhow::{anyhow, Result};

/// Validate basic eDNA abundance table invariants.
///
/// # Errors
/// Returns an error if rows are empty, sample IDs are inconsistent, or required columns are missing.
pub fn validate_edna_table(
    rows: &[serde_json::Value],
    expected_columns: &[&str],
) -> Result<()> {
    if rows.is_empty() {
        return Err(anyhow!("eDNA output table is empty"));
    }
    for row in rows {
        let Some(obj) = row.as_object() else {
            return Err(anyhow!("eDNA output row is not an object"));
        };
        for col in expected_columns {
            if !obj.contains_key(*col) {
                return Err(anyhow!("eDNA output missing expected column `{col}`"));
            }
        }
    }
    let sample_ids = rows
        .iter()
        .filter_map(|row| row.get("sample_id").and_then(serde_json::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    if sample_ids.is_empty() {
        return Err(anyhow!("eDNA output has no sample_id values"));
    }
    Ok(())
}
