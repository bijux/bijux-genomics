use super::{VcfLocalContainerSmokeRow, VcfSmokeInputs};
use crate::evidence::registry::{load_vcf_registry_snapshot, smoke::VcfRegistrySmokeRecord};
use anyhow::{anyhow, ensure, Result};
use bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path},
};
/// Project retained host, container and external-delivery declarations without executing tools.
///
/// # Errors
/// Rejects inconsistent scope, missing registry/wrapper coverage and invalid host support paths.
pub fn collect_vcf_local_container_smoke_rows(
    repo_root: &Path,
    inputs: &VcfSmokeInputs<'_>,
) -> Result<Vec<VcfLocalContainerSmokeRow>> {
    super::admission::admit_scope(inputs)?;
    let registry = load_vcf_registry_snapshot(repo_root)?.smoke_records();
    let hosts = observe_host_support(repo_root, inputs)?;
    let retained_bindings = inputs
        .retained
        .iter()
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();
    let mut rows = inputs
        .retained
        .iter()
        .map(|row| build_row(repo_root, row, inputs, &registry, &hosts))
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    super::contract::ensure_vcf_local_container_smoke_contract(
        repo_root,
        &rows,
        &retained_bindings,
        hosts.len(),
    )?;
    Ok(rows)
}
fn observe_host_support(
    repo_root: &Path,
    inputs: &VcfSmokeInputs<'_>,
) -> Result<BTreeMap<String, String>> {
    let mut hosts = BTreeMap::new();
    for stage in inputs.stage_defaults.keys() {
        if let Some(path) = (inputs.host_support)(repo_root, stage)? {
            ensure!(
                !path.is_empty()
                    && Path::new(&path)
                        .components()
                        .all(|part| matches!(part, Component::Normal(_) | Component::CurDir)),
                "VCF host smoke support path must be a nonempty repository-relative path"
            );
            ensure!(
                repo_root.join(&path).is_file(),
                "VCF host smoke support must identify an available file"
            );
            hosts.insert(stage.clone(), path);
        }
    }
    Ok(hosts)
}
fn build_row(
    repo_root: &Path,
    retained_row: &VcfActiveStageToolMatrixRow,
    inputs: &VcfSmokeInputs<'_>,
    registry: &BTreeMap<String, VcfRegistrySmokeRecord>,
    hosts: &BTreeMap<String, String>,
) -> Result<VcfLocalContainerSmokeRow> {
    let registry_row = registry.get(&retained_row.tool_id).ok_or_else(|| {
        anyhow!(
            "VCF local-container smoke is missing registry coverage for retained tool `{}`",
            retained_row.tool_id
        )
    })?;
    if inputs.external_tool_ids.contains(&retained_row.tool_id) {
        return Ok(super::rows::external_row(retained_row, registry_row));
    }
    let is_governed_default = inputs
        .stage_defaults
        .get(&retained_row.stage_id)
        .is_some_and(|tool| tool == &retained_row.tool_id);
    if is_governed_default {
        if let Some(path) = hosts.get(&retained_row.stage_id) {
            return super::rows::host_row(retained_row, registry_row, path.clone());
        }
    }
    super::rows::container_row(repo_root, retained_row, registry_row, is_governed_default)
}
