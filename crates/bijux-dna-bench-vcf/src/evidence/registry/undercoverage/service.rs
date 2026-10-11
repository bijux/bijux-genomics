use super::model::VcfUndercoveredStagesReport;
use super::projection::collect_vcf_undercovered_stage_rows;
use super::serialization::render_vcf_undercovered_stages_tsv;
use super::VCF_UNDERCOVERED_STAGES_SCHEMA_VERSION;
use crate::inputs::VcfAdapterInputs;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
/// Publish the audit from admitted catalog bindings and registry observations.
///
/// # Errors
/// Rejects malformed declarations, audit contract failures and publication errors.
pub fn render_vcf_undercovered_stages(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfUndercoveredStagesReport> {
    let registry = super::super::load_vcf_registry_snapshot(repo_root)?;
    let serving_map = crate::evidence::serving::render_vcf_tool_serving_map(
        inputs,
        repo_root,
        Path::new(crate::evidence::serving::DEFAULT_VCF_TOOL_SERVING_MAP_PATH),
    )?;
    let output_path = repo_relative_path(repo_root, output_path);
    let stage_count = inputs.catalog().len();
    let rows = collect_vcf_undercovered_stage_rows(&registry, &serving_map)?;

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_vcf_undercovered_stages_tsv(&rows).as_bytes(),
    )?;

    let mut decision_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *decision_counts.entry(row.decision.clone()).or_default() += 1;
    }

    Ok(VcfUndercoveredStagesReport {
        schema_version: VCF_UNDERCOVERED_STAGES_SCHEMA_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        undercovered_stage_count: rows.len(),
        decision_counts,
        rows,
    })
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
