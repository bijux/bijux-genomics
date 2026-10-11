use super::model::VcfAdmixtureReadyReport;
use super::{
    COVERAGE_STATUS_COMPLETE, GOVERNED_STAGE_CONTRACT_SMOKE_TOOL_ID, REQUIRED_POPULATION_COUNT,
    REQUIRED_SAMPLE_COUNT, REQUIRED_SELECTED_K, VCF_ADMIXTURE_STAGE_ID,
};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
pub(super) fn ensure_vcf_admixture_ready_contract(report: &VcfAdmixtureReadyReport) -> Result<()> {
    const EXPECTED_TOOL_IDS: [&str; 2] = ["plink", "plink2"];
    if report.retained_row_count != EXPECTED_TOOL_IDS.len()
        || report.active_row_count != EXPECTED_TOOL_IDS.len()
        || report.complete_row_count != EXPECTED_TOOL_IDS.len()
        || report.incomplete_row_count != 0
        || report.violation_count != 0
        || !report.ok
    {
        return Err(anyhow!(
            "vcf.admixture readiness must retain every complete active governed backend"
        ));
    }
    let observed_tool_ids =
        report.rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
    if observed_tool_ids != EXPECTED_TOOL_IDS.into_iter().collect::<BTreeSet<_>>() {
        return Err(anyhow!(
            "vcf.admixture readiness backend set drifted from the governed catalog"
        ));
    }
    if report.rows.iter().any(|row| {
        row.stage_id != VCF_ADMIXTURE_STAGE_ID
            || row.coverage_status != COVERAGE_STATUS_COMPLETE
            || !row.command_ready
            || !row.output_ready
            || !row.parser_ready
            || !row.expected_result_ready
            || !row.report_ready
            || !row.smoke_ready
            || row.smoke_tool_id != GOVERNED_STAGE_CONTRACT_SMOKE_TOOL_ID
    }) {
        return Err(anyhow!(
            "vcf.admixture readiness rows drifted from the governed active backend contracts"
        ));
    }
    if report.rows.iter().any(|row| {
        row.report_section_id != "population_structure"
            || row.summary_table_id != "population_structure_metrics"
            || row.smoke_selected_k != REQUIRED_SELECTED_K
            || row.smoke_sample_count != REQUIRED_SAMPLE_COUNT
            || row.smoke_population_count != REQUIRED_POPULATION_COUNT
    }) {
        return Err(anyhow!(
            "vcf.admixture readiness row is missing required report or smoke invariants"
        ));
    }
    Ok(())
}
