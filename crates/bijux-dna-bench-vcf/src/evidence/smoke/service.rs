use super::{
    paths::{path_relative_to_repo, repo_relative_path},
    projection::collect_vcf_local_container_smoke_rows,
    serialization::render_vcf_local_container_smoke_tsv,
    VcfLocalContainerSmokeReport, VCF_LOCAL_CONTAINER_SMOKE_SCHEMA_VERSION,
};
use anyhow::{Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
/// Publish complete local smoke-path declarations after admission.
///
/// # Errors
/// Rejects declaration drift, unavailable wrappers or report publication failures.
pub fn render_vcf_local_container_smoke(
    repo_root: &Path,
    output_path: &Path,
    inputs: &super::VcfSmokeInputs<'_>,
) -> Result<VcfLocalContainerSmokeReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_vcf_local_container_smoke_rows(repo_root, inputs)?;

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::write_string(&output_path, &render_vcf_local_container_smoke_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut runtime_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *runtime_counts.entry(row.smoke_runtime.clone()).or_default() += 1;
    }
    let host_stage_smoke_row_count =
        rows.iter().filter(|row| row.smoke_path_kind == "host_stage_smoke").count();
    let container_smoke_row_count = rows
        .iter()
        .filter(|row| {
            matches!(
                row.smoke_path_kind.as_str(),
                "docker_container_smoke" | "apptainer_container_smoke"
            )
        })
        .count();
    let external_artifact_row_count =
        rows.iter().filter(|row| row.smoke_path_kind == "external_artifact_admission").count();

    Ok(VcfLocalContainerSmokeReport {
        schema_version: VCF_LOCAL_CONTAINER_SMOKE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        stage_count: rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len(),
        tool_count: rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len(),
        host_stage_smoke_row_count,
        container_smoke_row_count,
        external_artifact_row_count,
        runtime_counts,
        rows,
    })
}
