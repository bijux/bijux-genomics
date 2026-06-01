//! FASTQ domain helpers for v1.

use anyhow::{anyhow, Result};
use bijux_dna_core::ids::StageId;
use std::path::{Path, PathBuf};

pub use bijux_dna_planner_fastq::stage_api as fastq_banks;
pub use bijux_dna_planner_fastq::stage_api::args as fastq_args;
pub use bijux_dna_planner_fastq::stage_api::banks as fastq_bank_ops;
pub use bijux_dna_planner_fastq::stage_api::*;

pub use crate::internal::public_bridge::handlers::fastq::*;

/// # Errors
/// Returns an error when the stage does not expose the requested benchmark cohort.
pub fn benchmark_tools_for_stage(stage_id: &str, scenario_id: Option<&str>) -> Result<Vec<String>> {
    let stage_id = StageId::new(stage_id.to_string());
    let tool_ids = if let Some(scenario_id) = scenario_id {
        toolset_for_stage_benchmark_scenario(&stage_id, scenario_id)
    } else {
        benchmark_default_scenario_toolset(&stage_id)
    };
    if tool_ids.is_empty() {
        return if let Some(scenario_id) = scenario_id {
            Err(anyhow!(
                "stage `{}` does not expose benchmark cohort `{scenario_id}`",
                stage_id.as_str()
            ))
        } else {
            Err(anyhow!(
                "stage `{}` does not expose a unique default benchmark cohort",
                stage_id.as_str()
            ))
        };
    }
    Ok(tool_ids.into_iter().map(|tool_id| tool_id.to_string()).collect())
}

/// Materialize the governed local-ready `fastq.index_reference` dry-run plan.
///
/// The written artifact lives at `target/local-ready/fastq.index_reference/plan.json` under the
/// active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed planner config is
/// invalid, or the plan artifact cannot be written.
pub fn write_local_index_reference_plan() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let plan = bijux_dna_planner_fastq::stage_api::local_index_reference_plan(&repo_root)?;
    let plan_dir = resolve_plan_dir(&repo_root, &plan.out_dir);
    bijux_dna_infra::ensure_dir(&plan_dir)?;
    let plan_path = plan_dir.join("plan.json");
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
    Ok(plan_path)
}

/// Materialize the governed local-ready `fastq.deplete_rrna` dry-run plan.
///
/// The written artifact lives at `target/local-ready/fastq.deplete_rrna/plan.json` under the
/// active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed planner config is
/// invalid, or the plan artifact cannot be written.
pub fn write_local_deplete_rrna_plan() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let plan = bijux_dna_planner_fastq::stage_api::local_deplete_rrna_plan(&repo_root)?;
    let plan_dir = resolve_plan_dir(&repo_root, &plan.out_dir);
    bijux_dna_infra::ensure_dir(&plan_dir)?;
    let plan_path = plan_dir.join("plan.json");
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
    Ok(plan_path)
}

/// Materialize the governed local-smoke `fastq.validate_reads` report bundle.
///
/// The written summary artifact lives at `target/local-smoke/fastq.validate_reads/report.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_validate_reads_smoke_report() -> Result<PathBuf> {
    crate::internal::fastq::stages::validate_reads::write_local_validate_reads_smoke_report()
}

/// Materialize the governed local-smoke `fastq.profile_read_lengths` summary TSV.
///
/// The written summary artifact lives at `target/local-smoke/fastq.profile_read_lengths/read_lengths.tsv`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_profile_read_lengths_smoke_summary() -> Result<PathBuf> {
    crate::internal::fastq::stages::profile_read_lengths::write_local_profile_read_lengths_smoke_summary()
}

/// Materialize the governed local-smoke `fastq.profile_reads` report bundle.
///
/// The written summary artifact lives at `target/local-smoke/fastq.profile_reads/profile.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_profile_reads_smoke_report() -> Result<PathBuf> {
    crate::internal::fastq::stages::profile_reads::write_local_profile_reads_smoke_report()
}

/// Materialize the governed local-smoke `fastq.detect_adapters` report bundle.
///
/// The written summary artifact lives at `target/local-smoke/fastq.detect_adapters/adapters.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_detect_adapters_smoke_report() -> Result<PathBuf> {
    crate::internal::fastq::stages::detect_adapters::write_local_detect_adapters_smoke_report()
}

/// Materialize the governed local-smoke `fastq.filter_reads` artifacts.
///
/// The written summary artifact lives at `target/local-smoke/fastq.filter_reads/report.json`
/// under the active repository root, alongside the top-level `filtered.fastq.gz`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_filter_reads_smoke_report() -> Result<PathBuf> {
    crate::internal::fastq::stages::filter_reads::write_local_filter_reads_smoke_report()
}

/// Materialize the governed local-smoke `fastq.trim_reads` report bundle.
///
/// The written summary artifact lives at `target/local-smoke/fastq.trim_reads/report.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_trim_reads_smoke_report() -> Result<PathBuf> {
    crate::internal::fastq::stages::trim_reads::write_local_trim_reads_smoke_report()
}

/// Materialize the governed local-smoke `fastq.detect_duplicates_premerge` report bundle.
///
/// The written summary artifact lives at
/// `target/local-smoke/fastq.detect_duplicates_premerge/duplicates.json` under the active
/// repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_detect_duplicates_premerge_smoke_report() -> Result<PathBuf> {
    crate::internal::fastq::stages::detect_duplicates_premerge::write_local_detect_duplicates_premerge_smoke_report()
}

/// Materialize the governed local-smoke
/// `fastq.estimate_library_complexity_prealign` report bundle.
///
/// The written summary artifact lives at
/// `target/local-smoke/fastq.estimate_library_complexity_prealign/complexity.json` under the
/// active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_estimate_library_complexity_prealign_smoke_report() -> Result<PathBuf> {
    crate::internal::fastq::stages::estimate_library_complexity_prealign::write_local_estimate_library_complexity_prealign_smoke_report()
}

/// Materialize the governed local-smoke `fastq.normalize_primers` artifacts.
///
/// The written summary artifact lives at `target/local-smoke/fastq.normalize_primers/report.json`
/// under the active repository root, alongside the top-level `normalized.fastq.gz`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_normalize_primers_smoke_report() -> Result<PathBuf> {
    crate::internal::fastq::stages::normalize_primers::write_local_normalize_primers_smoke_report()
}

/// Materialize the governed local-smoke `fastq.trim_terminal_damage` artifacts.
///
/// The written summary artifact lives at `target/local-smoke/fastq.trim_terminal_damage/metrics.json`
/// under the active repository root, alongside the top-level `trimmed.fastq.gz`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_trim_terminal_damage_smoke_report() -> Result<PathBuf> {
    crate::internal::fastq::stages::trim_terminal_damage::write_local_trim_terminal_damage_smoke_report()
}

/// Materialize the governed local-smoke `fastq.trim_polyg_tails` artifacts.
///
/// The written summary artifact lives at `target/local-smoke/fastq.trim_polyg_tails/metrics.json`
/// under the active repository root, alongside the top-level `trimmed.fastq.gz`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_trim_polyg_tails_smoke_report() -> Result<PathBuf> {
    crate::internal::fastq::stages::trim_polyg_tails::write_local_trim_polyg_tails_smoke_report()
}

fn resolve_plan_dir(repo_root: &Path, out_dir: &Path) -> PathBuf {
    if out_dir.is_absolute() {
        out_dir.to_path_buf()
    } else {
        repo_root.join(out_dir)
    }
}
