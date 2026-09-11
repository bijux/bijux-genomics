use super::super::state::ReadinessState;
use crate::adapters::observations::VcfAdapterObservations;
use crate::evidence::output_coverage::{
    render_vcf_adapter_output_coverage, DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH,
};
use anyhow::{bail, Result};
use bijux_dna_bench_vcf_evidence::model::outputs::VcfAdapterOutputCoverageStatus;
use std::path::Path;
pub(super) fn output_coverage(
    run_id: &str,
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
) -> Result<String> {
    let report = render_vcf_adapter_output_coverage(
        observations,
        run_id,
        Path::new(DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH),
    )?;
    if report.benchmark_ready_row_count != state.counts.benchmark_ready
        || report.benchmark_ready_complete_row_count != state.counts.benchmark_ready
        || report.benchmark_ready_incomplete_row_count != 0
        || report.complete_row_count + report.incomplete_row_count != report.row_count
    {
        bail!("VCF adapter output coverage report drifted from the governed completeness contract");
    }
    state.counts.output_complete = report
        .rows
        .iter()
        .filter(|row| {
            row.benchmark_status == "benchmark_ready"
                && row.status == VcfAdapterOutputCoverageStatus::Complete
        })
        .count();
    state.adapter_output_coverage_report = Some(report);
    Ok("validated complete raw-output and normalized-output declarations for the benchmark-ready VCF slice".to_string())
}
