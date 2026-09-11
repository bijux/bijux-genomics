use super::{
    projection::collect_vcf_comparable_metric_rows,
    serialization::render_vcf_comparable_metrics_tsv,
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::{
    validate_vcf_comparable_metrics_report, VcfComparableMetricsReport,
    VCF_COMPARABLE_METRICS_SCHEMA_V1,
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
/// Publish governed comparison metrics from request-scoped adapter observations.
///
/// # Errors
/// Rejects unavailable adapters, uncovered multi-tool stages, invalid reports and IO failures.
pub fn render_vcf_comparable_metrics(
    observations: &crate::adapters::observations::VcfAdapterObservations<'_>,
    output_path: &Path,
) -> Result<VcfComparableMetricsReport> {
    let repo_root = observations.repo_root();
    let output_path = repo_relative_path(repo_root, output_path);
    let stage_tools = super::acquisition::collect_retained_tools_by_stage(observations)?;
    let rows = collect_vcf_comparable_metric_rows(&stage_tools)?;
    let multi_tool_stage_count = stage_tools.values().filter(|tools| tools.len() >= 2).count();
    let retained_tool_row_count =
        stage_tools.values().filter(|tools| tools.len() >= 2).map(BTreeSet::len).sum();
    let covered_stages =
        rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    if covered_stages != multi_tool_stage_count {
        return Err(anyhow!(
            "VCF comparable metrics must cover every multi-tool retained stage, covered {covered_stages} of {multi_tool_stage_count}"
        ));
    }

    let rendered = render_vcf_comparable_metrics_tsv(&rows);
    let report = VcfComparableMetricsReport {
        schema_version: VCF_COMPARABLE_METRICS_SCHEMA_V1.to_string(),
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count: covered_stages,
        multi_tool_stage_count,
        retained_tool_row_count,
        row_count: rows.len(),
        rows,
    };
    validate_vcf_comparable_metrics_report(&report)
        .context("validate VCF comparable-metrics report")?;
    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::write_bytes(&output_path, rendered.as_bytes())
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}
fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
