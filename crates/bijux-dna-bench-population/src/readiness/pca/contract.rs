use super::model::VcfPcaReadyReport;
use super::{COVERAGE_STATUS_COMPLETE, VCF_PCA_STAGE_ID};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
pub(super) fn ensure_vcf_pca_ready_contract(report: &VcfPcaReadyReport) -> Result<()> {
    if report.retained_row_count != report.rows.len() {
        return Err(anyhow!(
            "VCF PCA readiness must keep exactly one row per active retained `vcf.pca` binding"
        ));
    }
    if report.rows.is_empty() {
        return Err(anyhow!("VCF PCA readiness must keep at least one active retained caller row"));
    }
    if report.checked_surface_count != 8 {
        return Err(anyhow!("VCF PCA readiness must record exactly 8 checked surfaces"));
    }
    let unique_results =
        report.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    if unique_results != report.rows.len() {
        return Err(anyhow!(
            "VCF PCA readiness must keep one unique result_id per active retained caller row"
        ));
    }
    let observed_tools =
        report.rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
    let expected_tools = BTreeSet::from(["eigensoft", "plink2"]);
    if observed_tools != expected_tools {
        return Err(anyhow!(
            "VCF PCA readiness must retain the governed PCA tools `eigensoft` and `plink2`, found {observed_tools:?}"
        ));
    }
    for row in &report.rows {
        if row.stage_id != VCF_PCA_STAGE_ID {
            return Err(anyhow!(
                "VCF PCA readiness row `{}` drifted away from the `vcf.pca` stage",
                row.stage_id
            ));
        }
        if row.coverage_status == COVERAGE_STATUS_COMPLETE && !row.missing_surfaces.is_empty() {
            return Err(anyhow!(
                "VCF PCA readiness row `{}` / `{}` cannot be complete while listing missing surfaces",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}
