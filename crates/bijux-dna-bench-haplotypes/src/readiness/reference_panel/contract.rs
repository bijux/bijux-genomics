use super::model::VcfPrepareReferencePanelReadyReport;
use super::{
    COVERAGE_STATUS_COMPLETE, REQUIRED_NORMALIZATION_STATUS, REQUIRED_OUTPUT_VARIANT_COUNT_MINIMUM,
    VCF_PREPARE_REFERENCE_PANEL_STAGE_ID,
};
use anyhow::{anyhow, Result};
pub(super) fn ensure_vcf_prepare_reference_panel_ready_contract(
    report: &VcfPrepareReferencePanelReadyReport,
) -> Result<()> {
    if report.retained_row_count != 1
        || report.active_row_count != 1
        || report.complete_row_count != 1
        || report.incomplete_row_count != 0
        || report.checked_surface_count != 8
        || report.violation_count != 0
        || !report.ok
    {
        return Err(anyhow!(
            "vcf.prepare_reference_panel readiness counts drifted from the governed single-row active completion contract"
        ));
    }

    let row = report.rows.first().ok_or_else(|| {
        anyhow!("vcf.prepare_reference_panel readiness is missing its governed row")
    })?;
    if row.stage_id != VCF_PREPARE_REFERENCE_PANEL_STAGE_ID
        || row.tool_id != "bcftools"
        || row.report_section_id != "reference_panel_preparation"
        || row.summary_table_id != "reference_panel_readiness"
        || row.coverage_status != COVERAGE_STATUS_COMPLETE
        || row.smoke_normalization_status != REQUIRED_NORMALIZATION_STATUS
        || row.smoke_output_variants < REQUIRED_OUTPUT_VARIANT_COUNT_MINIMUM
        || !row.smoke_sample_consistent
        || !row.smoke_parseable
        || row.smoke_validation_checks.get("bgzip") != Some(&true)
        || row.smoke_validation_checks.get("tabix_index") != Some(&true)
        || row.smoke_validation_checks.get("sorted") != Some(&true)
    {
        return Err(anyhow!(
            "vcf.prepare_reference_panel readiness row drifted from the governed panel-preparation contract"
        ));
    }
    Ok(())
}
