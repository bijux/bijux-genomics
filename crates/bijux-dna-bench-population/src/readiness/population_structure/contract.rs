use super::model::VcfPopulationStructureReadyReport;
use super::{
    COVERAGE_STATUS_COMPLETE, GOVERNED_STAGE_CONTRACT_SMOKE_TOOL_ID, REQUIRED_SAMPLE_IDS,
    VCF_POPULATION_STRUCTURE_STAGE_ID,
};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
pub(super) fn ensure_vcf_population_structure_ready_contract(
    report: &VcfPopulationStructureReadyReport,
) -> Result<()> {
    const EXPECTED_TOOL_IDS: [&str; 3] = ["eigensoft", "plink", "plink2"];
    if report.retained_row_count != EXPECTED_TOOL_IDS.len()
        || report.active_row_count != EXPECTED_TOOL_IDS.len()
        || report.complete_row_count != EXPECTED_TOOL_IDS.len()
        || report.incomplete_row_count != 0
        || report.violation_count != 0
        || !report.ok
    {
        return Err(anyhow!(
            "vcf.population_structure readiness must retain every complete active governed backend"
        ));
    }
    let observed_tool_ids =
        report.rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
    if observed_tool_ids != EXPECTED_TOOL_IDS.into_iter().collect::<BTreeSet<_>>() {
        return Err(anyhow!(
            "vcf.population_structure readiness backend set drifted from the governed catalog"
        ));
    }
    if report.rows.iter().any(|row| {
        row.stage_id != VCF_POPULATION_STRUCTURE_STAGE_ID
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
            "vcf.population_structure readiness rows drifted from the governed active backend contracts"
        ));
    }
    if report.rows.iter().any(|row| {
        row.report_section_id != "population_structure"
            || row.summary_table_id != "population_structure_metrics"
            || row.smoke_sample_group_count != REQUIRED_SAMPLE_IDS.len()
    }) {
        return Err(anyhow!(
            "vcf.population_structure readiness row is missing required report or smoke invariants"
        ));
    }
    Ok(())
}
