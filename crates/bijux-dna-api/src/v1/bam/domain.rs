//! BAM local-ready and local-smoke domain helpers for v1.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Materialize the governed local-ready `bam.align` dry-run plan.
///
/// The written artifact lives at `target/local-ready/bam.align/plan.json` under the active
/// repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed planner config is
/// invalid, or the plan artifact cannot be written.
pub fn write_local_align_plan() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let plan = bijux_dna_planner_bam::stage_api::local_align_plan(&repo_root)?;
    let plan_dir = resolve_plan_dir(&repo_root, &plan.out_dir);
    bijux_dna_infra::ensure_dir(&plan_dir)?;
    let plan_path = plan_dir.join("plan.json");
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
    Ok(plan_path)
}

/// Materialize the governed local-smoke `bam.validate` report bundle.
///
/// The written summary artifact lives at `target/local-smoke/bam.validate/validation.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_validate_smoke_report() -> Result<PathBuf> {
    crate::internal::bam::stages::validate::write_local_validate_smoke_report()
}

/// Materialize the governed local-smoke `bam.qc_pre` report bundle.
///
/// The written summary artifact lives at `target/local-smoke/bam.qc_pre/qc_pre.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_qc_pre_smoke_report() -> Result<PathBuf> {
    crate::internal::bam::stages::qc_pre::write_local_qc_pre_smoke_report()
}

/// Materialize the governed local-smoke `bam.filter` report bundle.
///
/// The written metrics artifact lives at `target/local-smoke/bam.filter/filter_metrics.json`
/// under the active repository root, alongside the curated top-level `filtered.bam`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_filter_smoke_report() -> Result<PathBuf> {
    crate::internal::bam::stages::filter::write_local_filter_smoke_report()
}

/// Materialize the governed local-smoke `bam.duplication_metrics` report bundle.
///
/// The written report artifact lives at
/// `target/local-smoke/bam.duplication_metrics/duplication_metrics.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_duplication_metrics_smoke_report() -> Result<PathBuf> {
    crate::internal::bam::stages::duplication_metrics::write_local_duplication_metrics_smoke_report(
    )
}

/// Materialize the governed local-smoke `bam.complexity` report bundle.
///
/// The written report artifact lives at `target/local-smoke/bam.complexity/complexity.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_complexity_smoke_report() -> Result<PathBuf> {
    crate::internal::bam::stages::complexity::write_local_complexity_smoke_report()
}

/// Materialize the governed local-smoke `bam.insert_size` report bundle.
///
/// The written report artifact lives at `target/local-smoke/bam.insert_size/insert_size.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_insert_size_smoke_report() -> Result<PathBuf> {
    crate::internal::bam::stages::insert_size::write_local_insert_size_smoke_report()
}

/// Materialize the governed local-smoke `bam.gc_bias` TSV bundle.
///
/// The written summary artifact lives at `target/local-smoke/bam.gc_bias/gc_bias.tsv`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_gc_bias_smoke_summary() -> Result<PathBuf> {
    crate::internal::bam::stages::gc_bias::write_local_gc_bias_smoke_summary()
}

/// Materialize the governed local-smoke `bam.endogenous_content` report bundle.
///
/// The written report artifact lives at
/// `target/local-smoke/bam.endogenous_content/endogenous_content.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_endogenous_content_smoke_report() -> Result<PathBuf> {
    crate::internal::bam::stages::endogenous_content::write_local_endogenous_content_smoke_report()
}

/// Materialize the governed local-smoke `bam.coverage` TSV bundle.
///
/// The written summary artifact lives at
/// `target/local-smoke/bam.coverage/coverage.tsv`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_coverage_smoke_summary() -> Result<PathBuf> {
    crate::internal::bam::stages::coverage::write_local_coverage_smoke_summary()
}

/// Materialize the governed local-smoke `bam.length_filter` report bundle.
///
/// The written report artifact lives at `target/local-smoke/bam.length_filter/length_filter.json`
/// under the active repository root, alongside the curated top-level `length_filtered.bam`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_length_filter_smoke_report() -> Result<PathBuf> {
    crate::internal::bam::stages::length_filter::write_local_length_filter_smoke_report()
}

/// Materialize the governed local-smoke `bam.markdup` report bundle.
///
/// The written report artifact lives at `target/local-smoke/bam.markdup/duplicates.json`
/// under the active repository root, alongside the curated top-level `marked.bam`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_markdup_smoke_report() -> Result<PathBuf> {
    crate::internal::bam::stages::markdup::write_local_markdup_smoke_report()
}

/// Materialize the governed local-smoke `bam.mapq_filter` report bundle.
///
/// The written report artifact lives at `target/local-smoke/bam.mapq_filter/mapq_filter.json`
/// under the active repository root, alongside the curated top-level `mapq_filtered.bam`.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_mapq_filter_smoke_report() -> Result<PathBuf> {
    crate::internal::bam::stages::mapq_filter::write_local_mapq_filter_smoke_report()
}

/// Materialize the governed local-smoke `bam.mapping_summary` TSV bundle.
///
/// The written summary artifact lives at
/// `target/local-smoke/bam.mapping_summary/mapping_summary.tsv`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-smoke config is
/// invalid, or the smoke artifacts cannot be written.
pub fn write_local_mapping_summary_smoke_summary() -> Result<PathBuf> {
    crate::internal::bam::stages::mapping_summary::write_local_mapping_summary_smoke_summary()
}

fn resolve_plan_dir(repo_root: &Path, out_dir: &Path) -> PathBuf {
    if out_dir.is_absolute() {
        out_dir.to_path_buf()
    } else {
        repo_root.join(out_dir)
    }
}
