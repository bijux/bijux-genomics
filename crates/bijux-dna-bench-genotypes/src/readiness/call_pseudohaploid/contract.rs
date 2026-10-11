use super::model::VcfCallPseudohaploidReadyReport;
use super::{COVERAGE_STATUS_COMPLETE, VCF_CALL_PSEUDOHAPLOID_STAGE_ID};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
pub(super) fn ensure_vcf_call_pseudohaploid_ready_contract(
    report: &VcfCallPseudohaploidReadyReport,
) -> Result<()> {
    if report.retained_row_count != report.rows.len() {
        return Err(anyhow!(
            "VCF call_pseudohaploid readiness must keep exactly one row per retained `vcf.call_pseudohaploid` binding"
        ));
    }
    if report.rows.is_empty() {
        return Err(anyhow!(
            "VCF call_pseudohaploid readiness must keep at least one active retained caller row"
        ));
    }
    if report.checked_surface_count != 8 {
        return Err(anyhow!(
            "VCF call_pseudohaploid readiness must record exactly 8 checked surfaces"
        ));
    }
    let unique_results =
        report.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    if unique_results != report.rows.len() {
        return Err(anyhow!(
            "VCF call_pseudohaploid readiness must keep one unique result_id per retained caller row"
        ));
    }
    for row in &report.rows {
        if row.stage_id != VCF_CALL_PSEUDOHAPLOID_STAGE_ID {
            return Err(anyhow!(
                "VCF call_pseudohaploid readiness row `{}` drifted away from the `vcf.call_pseudohaploid` stage",
                row.stage_id
            ));
        }
        if row.coverage_status == COVERAGE_STATUS_COMPLETE && !row.missing_surfaces.is_empty() {
            return Err(anyhow!(
                "VCF call_pseudohaploid readiness row `{}` / `{}` cannot be complete while listing missing surfaces",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}
