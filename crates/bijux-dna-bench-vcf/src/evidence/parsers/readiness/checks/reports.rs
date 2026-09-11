use super::super::state::ReadinessState;
use crate::adapters::observations::VcfAdapterObservations;
use crate::evidence::reports::{render_vcf_report_map, DEFAULT_VCF_REPORT_MAP_PATH};
use anyhow::{bail, Result};
use std::path::Path;
pub(super) fn report_map(
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
) -> Result<String> {
    let report = render_vcf_report_map(observations, Path::new(DEFAULT_VCF_REPORT_MAP_PATH))?;
    if report.row_count != state.counts.expected_results
        || report.stage_count != 20
        || report.section_count != 12
        || report.summary_table_count != 12
    {
        bail!("VCF report map drifted from the governed expected-result slice");
    }
    state.counts.report_rows = report.row_count;
    state.report_map_report = Some(report);
    Ok("validated governed report-map coverage across the expected benchmark-ready VCF rows"
        .to_string())
}
