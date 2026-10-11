use super::placement::{failure_columns_for_stage, placement_for_section};
use anyhow::{anyhow, Result};
use bijux_dna_bench_vcf_evidence::model::reports::VcfReportMapRow;
use bijux_dna_domain_vcf::{find_vcf_parser_fixture_inventory_row, VcfDomainStage};

/// Project report placements from expected results admitted against the supplied inputs.
///
/// # Errors
/// Rejects mismatched expected bindings, parser inventory or report contracts.
pub fn collect_vcf_report_map_rows_from_expected(
    inputs: &crate::inputs::VcfAdapterInputs,
    expected_rows: Vec<
        bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow,
    >,
) -> Result<Vec<VcfReportMapRow>> {
    super::admission::validate_expected_rows(inputs, &expected_rows)?;
    let expected_row_count = expected_rows.len();
    let mut rows = Vec::with_capacity(expected_rows.len());

    for row in expected_rows {
        let stage = VcfDomainStage::try_from(row.stage_id.as_str())
            .map_err(|error| anyhow!("unknown VCF stage `{}`: {error}", row.stage_id))?;
        let placement = placement_for_section(row.report_section.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF report map is missing placement for benchmark section `{}`",
                row.report_section
            )
        })?;
        let parser_fixture = find_vcf_parser_fixture_inventory_row(&row.tool_id, stage)
            .ok_or_else(|| {
                anyhow!(
                    "VCF report map is missing parser fixture inventory for `{}` / `{}`",
                    row.stage_id,
                    row.tool_id
                )
            })?;

        if parser_fixture.fixture_path.trim().is_empty() {
            return Err(anyhow!(
                "VCF report map parser fixture inventory for `{}` / `{}` is missing a fixture path",
                row.stage_id,
                row.tool_id
            ));
        }

        rows.push(VcfReportMapRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            section_id: placement.section_id.to_string(),
            summary_table: placement.summary_table.to_string(),
            metric_columns: row.expected_metrics,
            failure_columns: failure_columns_for_stage(stage),
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.section_id.cmp(&right.section_id))
    });
    super::contract::ensure_vcf_report_map_contract(&rows, expected_row_count)?;
    Ok(rows)
}
