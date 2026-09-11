use super::model::{VcfMissingResultReport, VcfMissingResultStatus};
use anyhow::{anyhow, Result};
pub(super) fn ensure_vcf_missing_result_report_contract(
    mut report: VcfMissingResultReport,
) -> Result<VcfMissingResultReport> {
    if report.rows.len() != report.expected_row_count {
        return Err(anyhow!(
            "VCF missing-result report row count drifted from expected benchmark rows: expected {}, found {}",
            report.expected_row_count,
            report.rows.len(),
        ));
    }
    if report.present_result_row_count + report.missing_result_row_count
        != report.expected_row_count
    {
        return Err(anyhow!(
            "VCF missing-result report present/missing counts do not cover all {} expected rows",
            report.expected_row_count,
        ));
    }
    if report.present_result_row_count != report.expected_row_count.saturating_sub(1) {
        return Err(anyhow!(
            "VCF missing-result report must retain every expected row except the one removed result, expected {}, found {}",
            report.expected_row_count.saturating_sub(1),
            report.present_result_row_count,
        ));
    }
    if report.missing_result_row_count != 1 {
        return Err(anyhow!(
            "VCF missing-result report must emit exactly one missing_result row, found {}",
            report.missing_result_row_count
        ));
    }
    let removed_row = report
        .rows
        .iter()
        .find(|row| row.result_row_id == report.removed_result_row_id)
        .ok_or_else(|| {
            anyhow!("missing removed VCF result row `{}`", report.removed_result_row_id)
        })?;
    if removed_row.result_status != VcfMissingResultStatus::MissingResult
        || removed_row.domain != "vcf"
        || removed_row.stage_id != "vcf.stats"
        || removed_row.tool_id != "bcftools"
        || removed_row.corpus_id != "vcf_production_regression"
        || removed_row.asset_profile_id != "vcf_cohort"
    {
        return Err(anyhow!(
            "VCF missing-result report must keep the governed stats row visible as missing_result"
        ));
    }
    if report
        .rows
        .iter()
        .filter(|row| row.result_status == VcfMissingResultStatus::MissingResult)
        .count()
        != 1
    {
        return Err(anyhow!("VCF missing-result report must not hide or duplicate missing rows"));
    }
    report.passes_behavior_test = true;
    Ok(report)
}
