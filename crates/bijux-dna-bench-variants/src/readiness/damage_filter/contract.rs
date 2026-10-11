use super::model::VcfDamageFilterReadyReport;
use super::{COVERAGE_STATUS_COMPLETE, VCF_DAMAGE_FILTER_STAGE_ID};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
pub(super) fn ensure_vcf_damage_filter_ready_contract(
    report: &VcfDamageFilterReadyReport,
) -> Result<()> {
    if report.retained_row_count != report.rows.len() {
        return Err(anyhow!(
            "VCF damage_filter readiness must keep exactly one row per active retained `vcf.damage_filter` binding"
        ));
    }
    if report.rows.is_empty() {
        return Err(anyhow!(
            "VCF damage_filter readiness must keep at least one active retained caller row"
        ));
    }
    if report.checked_surface_count != 8 {
        return Err(anyhow!("VCF damage_filter readiness must record exactly 8 checked surfaces"));
    }
    let unique_results =
        report.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    if unique_results != report.rows.len() {
        return Err(anyhow!(
            "VCF damage_filter readiness must keep one unique result_id per active retained caller row"
        ));
    }
    for row in &report.rows {
        if row.stage_id != VCF_DAMAGE_FILTER_STAGE_ID {
            return Err(anyhow!(
                "VCF damage_filter readiness row `{}` drifted away from the `vcf.damage_filter` stage",
                row.stage_id
            ));
        }
        if row.coverage_status == COVERAGE_STATUS_COMPLETE && !row.missing_surfaces.is_empty() {
            return Err(anyhow!(
                "VCF damage_filter readiness row `{}` / `{}` cannot be complete while listing missing surfaces",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}
