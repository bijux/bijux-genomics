use anyhow::Result;

#[test]
fn edna_output_invariants_require_non_empty_rows_and_columns() -> Result<()> {
    let rows = vec![serde_json::json!({
        "sample_id": "s1",
        "feature_id": "asv1",
        "abundance": 0.15
    })];
    bijux_dna_domain_fastq::validate_edna_table(&rows, &["sample_id", "feature_id", "abundance"])?;
    Ok(())
}
