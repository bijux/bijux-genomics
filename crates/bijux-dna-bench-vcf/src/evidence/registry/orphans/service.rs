use super::model::VcfOrphanToolsReport;
use super::projection::collect_vcf_orphan_tool_rows;
use super::serialization::render_vcf_orphan_tools_tsv;
use super::VCF_ORPHAN_TOOLS_SCHEMA_VERSION;
use crate::inputs::VcfAdapterInputs;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
/// Publish the audit from admitted catalog bindings and registry observations.
///
/// # Errors
/// Rejects malformed declarations, audit contract failures and publication errors.
pub fn render_vcf_orphan_tools(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfOrphanToolsReport> {
    let registry = super::super::load_vcf_registry_snapshot(repo_root)?;
    let required_tool_ids = super::super::acquisition::load_required_vcf_tool_ids(repo_root)?;
    let serving_map = crate::evidence::serving::render_vcf_tool_serving_map(
        inputs,
        repo_root,
        Path::new(crate::evidence::serving::DEFAULT_VCF_TOOL_SERVING_MAP_PATH),
    )?;
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_vcf_orphan_tool_rows(inputs, &registry, &serving_map, &required_tool_ids)?;
    let required_tool_count = required_tool_ids.len();
    let registered_tool_count = registry.tool_records.len();
    let served_tool_count =
        serving_map.rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_vcf_orphan_tools_tsv(&rows).as_bytes(),
    )?;

    Ok(VcfOrphanToolsReport {
        schema_version: VCF_ORPHAN_TOOLS_SCHEMA_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        orphan_count: rows.len(),
        required_tool_count,
        registered_tool_count,
        served_tool_count,
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
