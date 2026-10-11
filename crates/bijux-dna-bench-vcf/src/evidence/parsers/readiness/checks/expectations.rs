use super::super::state::ReadinessState;
use crate::adapters::observations::VcfAdapterObservations;
use crate::evidence::expectations::{
    render_vcf_expected_benchmark_results, DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use anyhow::{bail, Result};
use std::path::Path;
pub(super) fn expected_results(
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
) -> Result<String> {
    let report = render_vcf_expected_benchmark_results(
        observations,
        Path::new(DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH),
    )?;
    if report.stage_count != 20 || report.corpus_count != 1 || report.asset_profile_count != 6 {
        bail!("VCF expected benchmark results drifted from the governed ready slice");
    }
    state.counts.expected_results = report.row_count;
    let row_count = report.row_count;
    state.expected_results_report = Some(report);
    Ok(format!("validated {row_count} governed expected benchmark-ready VCF result rows"))
}
