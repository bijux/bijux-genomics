use super::super::state::ReadinessState;
use crate::adapters::observations::VcfAdapterObservations;
use crate::evidence::metrics::{
    render_vcf_comparable_metrics, DEFAULT_VCF_COMPARABLE_METRICS_PATH,
};
use anyhow::{bail, Result};
use std::path::Path;
pub(super) fn comparable_metrics(
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
) -> Result<String> {
    let report = render_vcf_comparable_metrics(
        observations,
        Path::new(DEFAULT_VCF_COMPARABLE_METRICS_PATH),
    )?;
    if report.stage_count != 10
        || report.multi_tool_stage_count != 10
        || report.retained_tool_row_count != 29
        || report.row_count != 29
    {
        bail!("VCF comparable metrics report drifted from the governed retained slice");
    }
    state.counts.metrics = report.row_count;
    Ok("validated governed comparable metrics across the retained multi-tool VCF stage slice"
        .to_string())
}
