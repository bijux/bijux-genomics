use anyhow::{anyhow, Result};
use bijux_dna_bench_vcf_evidence::model::reports::VcfReportMapRow;
use std::collections::BTreeSet;
pub(super) fn ensure_vcf_report_map_contract(
    rows: &[VcfReportMapRow],
    expected_row_count: usize,
) -> Result<()> {
    let unique_pairs =
        rows.iter().map(|row| (&row.stage_id, &row.tool_id)).collect::<BTreeSet<_>>();
    if unique_pairs.len() != rows.len() {
        return Err(anyhow!(
            "VCF report map must keep one row per expected VCF stage-tool result binding"
        ));
    }
    if rows.len() != expected_row_count {
        return Err(anyhow!(
            "VCF report map must retain every benchmark-ready row, expected {expected_row_count}, found {}",
            rows.len(),
        ));
    }

    for row in rows {
        if row.section_id.trim().is_empty()
            || row.summary_table.trim().is_empty()
            || row.metric_columns.is_empty()
            || row.failure_columns.is_empty()
        {
            return Err(anyhow!(
                "VCF report map row `{}` / `{}` is missing required report columns",
                row.stage_id,
                row.tool_id
            ));
        }
    }

    let section_count =
        rows.iter().map(|row| row.section_id.as_str()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table.as_str()).collect::<BTreeSet<_>>().len();
    if section_count != 12 || summary_table_count != 12 {
        return Err(anyhow!(
            "VCF report map must retain 12 sections and 12 summary tables for the governed ready slice, found {section_count} sections and {summary_table_count} tables"
        ));
    }

    super::governed::require_governed_mappings(rows)?;

    Ok(())
}
