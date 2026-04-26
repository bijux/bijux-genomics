use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::{FastqStageBinding, STAGE_REPORT_QC};

use super::input_resolution::stage_node_id_for_binding;
use super::models::{PlannedStageLineage, ReferenceIndexState, StageDependencyPolicy};

pub(super) fn inherited_lineage(
    binding: &FastqStageBinding,
    stage_dependencies: Option<&StageDependencyPolicy>,
    lineage_by_node_id: &BTreeMap<String, PlannedStageLineage>,
    latest_lineage_node_id: Option<&str>,
    raw_r1: &Path,
    raw_r2: Option<&Path>,
) -> Result<PlannedStageLineage> {
    let upstream_lineages =
        upstream_lineages(binding, stage_dependencies, lineage_by_node_id, latest_lineage_node_id);
    if upstream_lineages.is_empty() {
        return Ok(PlannedStageLineage {
            reads_r1: raw_r1.to_path_buf(),
            reads_r2: raw_r2.map(Path::to_path_buf),
            feature_table: None,
            reference_index: None,
            qc_inputs: Vec::new(),
            lineage_inputs: Vec::new(),
        });
    }

    if binding.stage_id == STAGE_REPORT_QC.as_str() {
        let qc_inputs = combine_qc_inputs(&upstream_lineages);
        return Ok(PlannedStageLineage {
            reads_r1: raw_r1.to_path_buf(),
            reads_r2: raw_r2.map(Path::to_path_buf),
            feature_table: None,
            reference_index: None,
            qc_inputs,
            lineage_inputs: combine_lineage_inputs(&upstream_lineages),
        });
    }

    let reads_r1 = unique_required_path_for_binding(
        binding,
        "reads_r1",
        upstream_lineages.iter().map(|lineage| lineage.reads_r1.clone()).collect(),
    )?;
    let reads_r2 = unique_optional_path_for_binding(
        binding,
        "reads_r2",
        upstream_lineages.iter().map(|lineage| lineage.reads_r2.clone()).collect(),
    )?;
    let feature_table = unique_optional_path_for_binding(
        binding,
        "abundance_table",
        upstream_lineages.iter().map(|lineage| lineage.feature_table.clone()).collect(),
    )?;
    let reference_index = unique_reference_index_for_binding(
        binding,
        upstream_lineages.iter().map(|lineage| lineage.reference_index.clone()).collect(),
    )?;
    Ok(PlannedStageLineage {
        reads_r1,
        reads_r2,
        feature_table,
        reference_index,
        qc_inputs: combine_qc_inputs(&upstream_lineages),
        lineage_inputs: combine_lineage_inputs(&upstream_lineages),
    })
}

pub(super) fn merge_lineage_input_artifacts(
    plan: &mut StagePlanV1,
    lineage_inputs: &[ArtifactRef],
) {
    for artifact in lineage_inputs {
        if plan
            .io
            .inputs
            .iter()
            .any(|existing| existing.name == artifact.name && existing.path == artifact.path)
        {
            continue;
        }
        plan.io.inputs.push(artifact.clone());
    }
    plan.io.inputs.sort_by(|left, right| {
        left.name.as_str().cmp(right.name.as_str()).then_with(|| left.path.cmp(&right.path))
    });
}

pub(super) fn lineage_input_artifacts_for_stage(plan: &StagePlanV1) -> Vec<ArtifactRef> {
    plan.io
        .outputs
        .iter()
        .filter(|artifact| artifact.name.as_str() == "validated_reads_manifest")
        .cloned()
        .collect()
}

fn combine_qc_inputs(upstream_lineages: &[&PlannedStageLineage]) -> Vec<ArtifactRef> {
    let mut qc_inputs =
        upstream_lineages.iter().flat_map(|lineage| lineage.qc_inputs.clone()).collect::<Vec<_>>();
    qc_inputs.sort_by(|left, right| {
        left.name.as_str().cmp(right.name.as_str()).then_with(|| left.path.cmp(&right.path))
    });
    qc_inputs.dedup_by(|left, right| left.name == right.name && left.path == right.path);
    qc_inputs
}

fn combine_lineage_inputs(upstream_lineages: &[&PlannedStageLineage]) -> Vec<ArtifactRef> {
    let mut lineage_inputs = upstream_lineages
        .iter()
        .flat_map(|lineage| lineage.lineage_inputs.clone())
        .collect::<Vec<_>>();
    lineage_inputs.sort_by(|left, right| {
        left.name.as_str().cmp(right.name.as_str()).then_with(|| left.path.cmp(&right.path))
    });
    lineage_inputs.dedup_by(|left, right| left.name == right.name && left.path == right.path);
    lineage_inputs
}

fn upstream_lineages<'a>(
    binding: &FastqStageBinding,
    stage_dependencies: Option<&StageDependencyPolicy>,
    lineage_by_node_id: &'a BTreeMap<String, PlannedStageLineage>,
    latest_lineage_node_id: Option<&str>,
) -> Vec<&'a PlannedStageLineage> {
    let node_id = stage_node_id_for_binding(binding);
    if let Some(policy) = stage_dependencies {
        return policy
            .get(&node_id)
            .into_iter()
            .flat_map(|upstream_nodes| upstream_nodes.iter())
            .filter_map(|upstream_node| lineage_by_node_id.get(upstream_node))
            .collect();
    }
    latest_lineage_node_id.and_then(|node_id| lineage_by_node_id.get(node_id)).into_iter().collect()
}

fn unique_required_path_for_binding(
    binding: &FastqStageBinding,
    input_id: &str,
    paths: Vec<PathBuf>,
) -> Result<PathBuf> {
    let mut unique_paths = paths;
    unique_paths.sort();
    unique_paths.dedup();
    match unique_paths.len() {
        1 => Ok(unique_paths.remove(0)),
        0 => Err(anyhow!("{} is missing upstream {input_id} lineage", binding.stage_id)),
        _ => Err(anyhow!(
            "{} has multiple upstream candidates for {input_id}; add an explicit artifact binding",
            binding.stage_id
        )),
    }
}

fn unique_optional_path_for_binding(
    binding: &FastqStageBinding,
    input_id: &str,
    paths: Vec<Option<PathBuf>>,
) -> Result<Option<PathBuf>> {
    let mut unique_paths = paths.into_iter().flatten().collect::<Vec<_>>();
    unique_paths.sort();
    unique_paths.dedup();
    match unique_paths.len() {
        0 => Ok(None),
        1 => Ok(unique_paths.into_iter().next()),
        _ => Err(anyhow!(
            "{} has multiple upstream candidates for {input_id}; add an explicit artifact binding",
            binding.stage_id
        )),
    }
}

fn unique_reference_index_for_binding(
    binding: &FastqStageBinding,
    indices: Vec<Option<ReferenceIndexState>>,
) -> Result<Option<ReferenceIndexState>> {
    let mut unique_indices = indices.into_iter().flatten().collect::<Vec<_>>();
    unique_indices.sort_by(|left, right| {
        left.path.cmp(&right.path).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    unique_indices.dedup_by(|left, right| left.path == right.path && left.tool_id == right.tool_id);
    match unique_indices.len() {
        0 => Ok(None),
        1 => Ok(unique_indices.into_iter().next()),
        _ => Err(anyhow!(
            "{} has multiple upstream reference indexes; add an explicit reference_index artifact binding",
            binding.stage_id
        )),
    }
}
