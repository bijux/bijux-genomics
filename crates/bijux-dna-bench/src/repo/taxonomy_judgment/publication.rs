//! Owner: benchmark taxonomy judgment publication.
//! Existing benchmark taxonomy judgment publication behavior.
use super::{judge_edna_taxonomy_outputs, TaxonomyObservedReportArg};
use anyhow::{Context, Result};
use bijux_dna_bench_model::BenchmarkTaxonomyOutputJudgmentReport;
use std::fs;
use std::path::{Path, PathBuf};

/// Acquire a judgment and write its existing pretty JSON representation.
///
/// # Errors
/// Rejects output directory creation, source admission/evaluation, serialization
/// or file writing errors. Invalid semantic verdicts remain represented in the report.
pub fn publish_edna_taxonomy_output_judgment(
    repo_root: &Path,
    manifest_path: PathBuf,
    reports: Vec<TaxonomyObservedReportArg>,
    output_path: PathBuf,
) -> Result<BenchmarkTaxonomyOutputJudgmentReport> {
    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = judge_edna_taxonomy_outputs(repo_root, &manifest_path, &reports)?;
    fs::write(
        &absolute_output_path,
        serde_json::to_string_pretty(&report).context("serialize taxonomy output judgment")?,
    )
    .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(report)
}
