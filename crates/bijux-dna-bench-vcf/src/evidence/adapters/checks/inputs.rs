use crate::adapters::observations::VcfAdapterObservations;
use crate::evidence::inputs::{
    render_vcf_adapter_missing_input_audit_with_observations,
    DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH,
};
use anyhow::{bail, Result};
use std::path::Path;
pub(super) fn missing_inputs(observations: &VcfAdapterObservations<'_>) -> Result<String> {
    let report = render_vcf_adapter_missing_input_audit_with_observations(
        observations,
        Path::new(DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH),
    )?;
    if report.row_count != 10
        || report.passed_row_count != 10
        || report.failed_row_count != 0
        || report.adapter_row_count != 9
        || report.support_row_count != 1
    {
        bail!("VCF adapter missing-input report drifted from the governed required-role contract");
    }
    Ok("validated all 10 governed VCF missing-input roles before tool execution".to_string())
}
