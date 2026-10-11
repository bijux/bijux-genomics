use super::super::state::ReadinessState;
use crate::adapters::angsd::DEFAULT_VCF_ANGSD_ADAPTER_PATH;
use crate::adapters::bcftools::DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH;
use crate::adapters::observations::VcfAdapterObservations;
use anyhow::{bail, Result};
use std::path::Path;
pub(super) fn bcftools(
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
) -> Result<String> {
    let report = observations.publish_bcftools(Path::new(DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH))?;
    if report.row_count != 11
        || report.supported_row_count != 11
        || report.planned_row_count != 0
        || report.argv_valid_row_count != report.row_count
        || report.missing_input_test_passed_row_count != report.row_count
        || report.indexed_row_count != 9
    {
        bail!("VCF bcftools adapter report drifted from the governed retained row contract");
    }
    state.counts.adapter_complete += report
        .rows
        .iter()
        .filter(|row| {
            row.benchmark_status == "benchmark_ready"
                && row.argv_validation_passed
                && row.missing_input_test_passed
        })
        .count();
    Ok("validated executable governed bcftools adapter rows with passing missing-input probes"
        .to_string())
}

pub(super) fn angsd(observations: &VcfAdapterObservations<'_>) -> Result<String> {
    let report = observations.publish_angsd(Path::new(DEFAULT_VCF_ANGSD_ADAPTER_PATH))?;
    if report.row_count != 2
        || report.supported_stage_row_count != 2
        || report.benchmark_ready_row_count != 2
        || report.argv_valid_row_count != report.row_count
        || report.missing_input_test_passed_row_count != report.row_count
        || report.bam_list_row_count != 2
        || report.parser_output_row_count != report.row_count
    {
        bail!("VCF angsd adapter report drifted from the governed low-coverage row contract");
    }
    Ok("validated retained ANGSD VCF adapter rows and their missing-input behavior".to_string())
}
