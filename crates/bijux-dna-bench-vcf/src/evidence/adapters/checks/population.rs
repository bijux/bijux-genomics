use super::super::state::ReadinessState;
use crate::adapters::eigensoft::DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH;
use crate::adapters::observations::VcfAdapterObservations;
use crate::adapters::plink::{DEFAULT_VCF_PLINK2_ADAPTER_PATH, DEFAULT_VCF_PLINK_ADAPTER_PATH};
use anyhow::{bail, Result};
use std::path::Path;
pub(super) fn plink(
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
) -> Result<String> {
    let plink_report =
        observations.publish_plink("plink", Path::new(DEFAULT_VCF_PLINK_ADAPTER_PATH))?;
    let plink2_family_report =
        observations.publish_plink("plink2", Path::new(DEFAULT_VCF_PLINK2_ADAPTER_PATH))?;
    if plink_report.row_count != 3
        || plink_report.benchmark_ready_row_count != 3
        || plink_report.parser_output_row_count != plink_report.row_count
        || plink_report.normalized_metrics_row_count != plink_report.row_count
        || plink_report.raw_output_declared_row_count != plink_report.row_count
        || plink_report.missing_input_test_passed_row_count != plink_report.row_count
    {
        bail!("VCF plink adapter report drifted from the governed retained row contract");
    }
    if plink2_family_report.row_count != 5
        || plink2_family_report.benchmark_ready_row_count != 5
        || plink2_family_report.parser_output_row_count != plink2_family_report.row_count
        || plink2_family_report.normalized_metrics_row_count != plink2_family_report.row_count
        || plink2_family_report.raw_output_declared_row_count != plink2_family_report.row_count
        || plink2_family_report.missing_input_test_passed_row_count
            != plink2_family_report.row_count
    {
        bail!("VCF plink2 adapter report drifted from the governed benchmarked row contract");
    }
    state.counts.adapter_complete += plink_report
        .rows
        .iter()
        .filter(|row| {
            row.benchmark_status == "benchmark_ready"
                && row.argv_validation_passed
                && row.missing_input_test_passed
        })
        .count();
    state.counts.adapter_complete += plink2_family_report
        .rows
        .iter()
        .filter(|row| {
            row.benchmark_status == "benchmark_ready"
                && row.argv_validation_passed
                && row.missing_input_test_passed
        })
        .count();
    Ok("validated governed plink and plink2 adapter rows with explicit raw and normalized outputs"
        .to_string())
}

pub(super) fn eigensoft(observations: &VcfAdapterObservations<'_>) -> Result<String> {
    let report = observations.publish_eigensoft(Path::new(DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH))?;
    if report.row_count != 2
        || report.benchmark_ready_row_count != 2
        || report.conversion_output_row_count != report.row_count
        || report.pca_output_row_count != report.row_count
        || report.missing_input_test_passed_row_count != report.row_count
    {
        bail!("VCF eigensoft adapter report drifted from the governed retained row contract");
    }
    Ok("validated retained EIGENSOFT VCF adapter rows and explicit convertf/smartpca outputs"
        .to_string())
}
