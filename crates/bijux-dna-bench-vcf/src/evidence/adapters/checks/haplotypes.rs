use crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH;
use crate::adapters::imputation::DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH;
use crate::adapters::observations::VcfAdapterObservations;
use crate::adapters::phasing::{
    DEFAULT_VCF_BEAGLE_ADAPTER_PATH, DEFAULT_VCF_EAGLE_ADAPTER_PATH,
    DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH, DEFAULT_VCF_SHAPEIT_ADAPTER_PATH,
};
use anyhow::{anyhow, bail, Result};
use std::path::Path;
pub(super) fn phasing(observations: &VcfAdapterObservations<'_>) -> Result<String> {
    let mut reports = Vec::new();
    for (tool_id, path) in [
        ("shapeit", DEFAULT_VCF_SHAPEIT_ADAPTER_PATH),
        ("shapeit5", DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH),
        ("eagle", DEFAULT_VCF_EAGLE_ADAPTER_PATH),
        ("beagle", DEFAULT_VCF_BEAGLE_ADAPTER_PATH),
    ] {
        reports.push(observations.publish_phasing(tool_id, Path::new(path))?);
    }
    for report in reports.iter().filter(|report| report.tool_id != "beagle") {
        if report.row_count != 1
            || report.benchmark_ready_row_count != 1
            || report.parser_output_row_count != 1
            || report.indexed_row_count != 1
            || report.missing_input_test_passed_row_count != 1
        {
            bail!("VCF phasing adapter report for `{}` drifted from the governed retained row contract", report.tool_id);
        }
    }
    let beagle_report = reports
        .iter()
        .find(|report| report.tool_id == "beagle")
        .ok_or_else(|| anyhow!("VCF phasing family is missing beagle"))?;
    if beagle_report.row_count != 3
        || beagle_report.benchmark_ready_row_count != 3
        || beagle_report.parser_output_row_count != 3
        || beagle_report.indexed_row_count != 3
        || beagle_report.missing_input_test_passed_row_count != 3
    {
        bail!("VCF phasing adapter report for `beagle` drifted from the governed retained row contract");
    }
    Ok("validated executable phasing adapter rows for every supported backend".to_string())
}

pub(super) fn imputation(observations: &VcfAdapterObservations<'_>) -> Result<String> {
    let report =
        observations.publish_imputation(Path::new(DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH))?;
    if report.row_count != 8
        || report.tool_count != 4
        || report.benchmark_ready_row_count != 8
        || report.parser_output_row_count != report.row_count
        || report.missing_input_test_passed_row_count != report.row_count
    {
        bail!(
            "VCF imputation-family adapter report drifted from the governed retained row contract"
        );
    }
    Ok("validated retained imputation-family adapter rows with explicit parser outputs and missing-input probes".to_string())
}

pub(super) fn descent(observations: &VcfAdapterObservations<'_>) -> Result<String> {
    let report =
        observations.publish_descent(Path::new(DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH))?;
    if report.row_count != 5
        || report.tool_count != 5
        || report.benchmark_ready_row_count != 5
        || report.parser_output_row_count != report.row_count
        || report.missing_input_test_passed_row_count != report.row_count
    {
        bail!("VCF descent-family adapter report drifted from the governed retained row contract");
    }
    Ok("validated retained IBD, ROH, and demography adapter rows".to_string())
}
