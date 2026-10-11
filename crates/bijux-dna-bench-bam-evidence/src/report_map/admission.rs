use super::models::BamReportMapRow;
use crate::observations::ExpectedResultBinding;
use anyhow::{anyhow, Result};
use bijux_dna_core::id_catalog::{
    BAM_ALIGN, BAM_CONTAMINATION, BAM_DAMAGE, BAM_KINSHIP, BAM_RECALIBRATION, BAM_SEX,
};
use std::collections::BTreeSet;
pub(super) fn ensure_bam_report_map_contract(
    rows: &[BamReportMapRow],
    expected_rows: &[ExpectedResultBinding],
) -> Result<()> {
    let report_result_ids =
        rows.iter().map(|row| row.result_row_id.as_str()).collect::<BTreeSet<_>>();
    if report_result_ids.len() != rows.len() {
        return Err(anyhow!("BAM report map must keep one row per expected BAM result row"));
    }

    let expected_result_ids =
        expected_rows.iter().map(|row| row.result_row_id.as_str()).collect::<BTreeSet<_>>();
    if report_result_ids != expected_result_ids {
        return Err(anyhow!(
            "BAM report map must retain every BAM expected-result row; expected {} rows and found {} report rows",
            expected_rows.len(),
            rows.len()
        ));
    }

    let report_keys = rows.iter().map(report_result_key).collect::<BTreeSet<_>>();
    let expected_keys = expected_rows.iter().map(expected_result_key).collect::<BTreeSet<_>>();
    if report_keys != expected_keys {
        return Err(anyhow!(
            "BAM report map drifted from the governed BAM expected-result identity slice"
        ));
    }

    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    if stage_count != 24 {
        return Err(anyhow!(
            "BAM report map must retain 24 benchmark-ready stages, found {stage_count}"
        ));
    }

    for row in rows {
        if row.result_row_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_family_id.trim().is_empty()
            || row.fixture_id.trim().is_empty()
            || row.sample_scope.trim().is_empty()
            || row.support_status.trim().is_empty()
            || row.anchor_tool_id.trim().is_empty()
            || row.anchor_support_status.trim().is_empty()
            || row.report_section_id.trim().is_empty()
            || row.summary_table_id.trim().is_empty()
        {
            return Err(anyhow!(
                "BAM report map row `{}` / `{}` is missing required report binding fields",
                row.stage_id,
                row.tool_id
            ));
        }
    }

    let section_count =
        rows.iter().map(|row| row.report_section_id.as_str()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table_id.as_str()).collect::<BTreeSet<_>>().len();
    if section_count != 7 || summary_table_count != 7 {
        return Err(anyhow!(
            "BAM report map must retain 7 sections and 7 summary tables, found {section_count} sections and {summary_table_count} tables"
        ));
    }

    ensure_governed_mappings(rows)?;

    Ok(())
}
pub(super) fn report_result_key(row: &BamReportMapRow) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        row.result_row_id, row.stage_id, row.tool_id, row.corpus_family_id, row.fixture_id
    )
}
pub(super) fn expected_result_key(row: &ExpectedResultBinding) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        row.result_row_id, row.stage_id, row.tool_id, row.corpus_family_id, row.fixture_id
    )
}
#[allow(clippy::too_many_arguments)]
pub(super) fn require_row_mapping(
    rows: &[BamReportMapRow],
    stage_id: &str,
    tool_id: &str,
    section_id: &str,
    summary_table_id: &str,
    criticality: &str,
    support_status: &str,
    anchor_tool_id: &str,
    anchor_support_status: &str,
    workflow_branch_id: Option<&str>,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.stage_id == stage_id && row.tool_id == tool_id)
        .ok_or_else(|| anyhow!("BAM report map is missing `{stage_id}` / `{tool_id}`"))?;
    if row.report_section_id != section_id
        || row.summary_table_id != summary_table_id
        || row.criticality != criticality
        || row.support_status != support_status
        || row.anchor_tool_id != anchor_tool_id
        || row.anchor_support_status != anchor_support_status
        || row.workflow_branch_id.as_deref() != workflow_branch_id
    {
        return Err(anyhow!(
            "BAM report map row `{stage_id}` / `{tool_id}` drifted from its governed reporting contract"
        ));
    }
    Ok(())
}

fn ensure_governed_mappings(rows: &[BamReportMapRow]) -> Result<()> {
    require_row_mapping(
        rows,
        BAM_ALIGN,
        "bwa",
        "alignment_intake",
        "alignment_baseline",
        "optional",
        "supported",
        "bwa",
        "supported",
        None,
    )?;
    require_row_mapping(
        rows,
        BAM_DAMAGE,
        "mapdamage2",
        "ancient_signal",
        "damage_authenticity",
        "essential",
        "supported",
        "mapdamage2",
        "supported",
        Some("ancient_dna_authenticity"),
    )?;
    require_row_mapping(
        rows,
        BAM_CONTAMINATION,
        "schmutzi",
        "ancient_signal",
        "damage_authenticity",
        "essential",
        "supported",
        "schmutzi",
        "supported",
        Some("ancient_dna_authenticity"),
    )?;
    require_row_mapping(
        rows,
        BAM_SEX,
        "rxy",
        "sample_identity",
        "identity_inference",
        "essential",
        "supported",
        "rxy",
        "supported",
        Some("sample_identity"),
    )?;
    require_row_mapping(
        rows,
        BAM_RECALIBRATION,
        "gatk",
        "downstream_readiness",
        "variant_readiness",
        "essential",
        "supported",
        "gatk",
        "supported",
        Some("variant_readiness"),
    )?;
    require_row_mapping(
        rows,
        BAM_KINSHIP,
        "king",
        "sample_identity",
        "identity_inference",
        "essential",
        "supported",
        "king",
        "supported",
        Some("sample_identity"),
    )?;

    Ok(())
}
