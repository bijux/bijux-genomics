use super::bam_active_row_consistency::DEFAULT_BAM_ACTIVE_ROW_CONSISTENCY_PATH;
use super::bam_adapter_output_contract::DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH;
use super::bam_local_container_probe::DEFAULT_BAM_LOCAL_CONTAINER_SMOKE_PATH;
use super::bam_parser_fixture_coverage::DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH;
use super::bam_rendered_commands::DEFAULT_BAM_RENDERED_COMMANDS_PATH;
use super::bam_report_map::DEFAULT_BAM_REPORT_MAP_PATH;
use super::bam_science_thresholds_ready::DEFAULT_BAM_SCIENCE_THRESHOLDS_READY_PATH;
use super::expected_benchmark_results::DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH;
use crate::commands::benchmark::local_bam_micro_smoke_subset::DEFAULT_BAM_MICRO_SMOKE_SUMMARY_PATH;
use anyhow::{bail, Context, Result};
use bijux_dna_bench_bam_proofs::retained_tools::{EvidencePaths, RetainedContext};
use std::path::Path;
mod bindings;
mod qualification;
mod stages;
pub(super) fn context(root: &Path) -> Result<RetainedContext<'_>> {
    let active = crate::commands::support::workspace_root::resolve_repo_root()?;
    if active.canonicalize().context("resolve retained proof repository")?
        != root.canonicalize().context("resolve requested retained evidence root")?
    {
        bail!("retained BAM proof repository differs from the requested evidence root");
    }
    Ok(RetainedContext {
        repo_root: root,
        bindings: &bindings::BindingAdapter,
        qualification: &qualification::QualificationAdapter,
        stages: &stages::StageAdapter,
        paths: EvidencePaths {
            local_smoke: DEFAULT_BAM_LOCAL_CONTAINER_SMOKE_PATH,
            parser_coverage: DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH,
            rendered_commands: DEFAULT_BAM_RENDERED_COMMANDS_PATH,
            output_declarations: DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH,
            expected_results: DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
            report_map: DEFAULT_BAM_REPORT_MAP_PATH,
            active_row_consistency: DEFAULT_BAM_ACTIVE_ROW_CONSISTENCY_PATH,
            micro_smoke_summary: DEFAULT_BAM_MICRO_SMOKE_SUMMARY_PATH,
            science_thresholds: DEFAULT_BAM_SCIENCE_THRESHOLDS_READY_PATH,
        },
    })
}
#[cfg(test)]
mod tests;
