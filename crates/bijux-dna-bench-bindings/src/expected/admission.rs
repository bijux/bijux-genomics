use super::ExpectedBenchmarkResultRow;
use anyhow::{anyhow, Result};
use bijux_dna_core::id_catalog::{
    BAM_DAMAGE, BAM_KINSHIP, FASTQ_INDEX_REFERENCE, FASTQ_SCREEN, FASTQ_STATS_NEUTRAL,
};
use std::collections::BTreeSet;
pub(super) fn ensure_expected_result_contract(rows: &[ExpectedBenchmarkResultRow]) -> Result<()> {
    if rows.is_empty() {
        return Err(anyhow!("expected benchmark result table must retain benchmark-ready rows"));
    }
    let unique_result_row_ids =
        rows.iter().map(|row| row.result_row_id.clone()).collect::<BTreeSet<_>>();
    if unique_result_row_ids.len() != rows.len() {
        return Err(anyhow!(
            "expected benchmark result table must keep unique result_row_id values"
        ));
    }
    ensure_row(
        rows,
        "fastq",
        FASTQ_SCREEN,
        "kraken2",
        "corpus-02-edna-mini",
        "sample-set",
        Some("classification_report_json"),
    )?;
    ensure_row(
        rows,
        "fastq",
        FASTQ_INDEX_REFERENCE,
        "bowtie2_build",
        "reference-index-assets",
        "asset-set",
        Some("report_json"),
    )?;
    ensure_row(
        rows,
        "fastq",
        FASTQ_STATS_NEUTRAL,
        "seqkit_stats",
        "corpus-01-mini",
        "sample-set",
        Some("qc_json"),
    )?;
    ensure_row(
        rows,
        "bam",
        BAM_DAMAGE,
        "mapdamage2",
        "corpus-01-adna-damage-mini",
        "adna_damage_non_udg",
        Some("damage_report"),
    )?;
    ensure_row(
        rows,
        "bam",
        BAM_KINSHIP,
        "king",
        "corpus-01-kinship-mini",
        "sample-set",
        Some("kinship_report"),
    )?;
    Ok(())
}
fn ensure_row(
    rows: &[ExpectedBenchmarkResultRow],
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    fixture_id: &str,
    sample_scope: &str,
    normalized_metrics_output_id: Option<&str>,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.domain == domain && row.stage_id == stage_id && row.tool_id == tool_id)
        .ok_or_else(|| {
            anyhow!("missing expected benchmark result row for `{stage_id}` / `{tool_id}`")
        })?;
    if row.fixture_id != fixture_id
        || row.sample_scope != sample_scope
        || row.normalized_metrics_output_id.as_deref() != normalized_metrics_output_id
    {
        return Err(anyhow!(
            "expected benchmark result row `{stage_id}` / `{tool_id}` drifted from its governed result contract"
        ));
    }
    Ok(())
}
