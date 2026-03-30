use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::ArtifactRef;
use bijux_dna_stage_contract::StagePlanV1;

use super::models::{ReferenceIndexState, ResolvedStageInputArtifact};
use super::{FastqStageBinding, StageArtifactInputPolicy, SyntheticStageArtifactPolicy};

pub(super) fn stage_node_id_for_binding(binding: &FastqStageBinding) -> String {
    binding
        .stage_instance_id
        .clone()
        .unwrap_or_else(|| binding.stage_id.clone())
}

pub(super) fn stage_node_id_for_plan(plan: &StagePlanV1) -> &str {
    plan.stage_instance_id
        .as_ref()
        .map(|step_id| step_id.as_str())
        .unwrap_or(plan.stage_id.as_str())
}

pub(super) fn resolved_stage_input_artifacts(
    binding: &FastqStageBinding,
    explicit_stage_inputs: Option<&StageArtifactInputPolicy>,
    synthetic_stage_artifacts: Option<&SyntheticStageArtifactPolicy>,
    plans: &[StagePlanV1],
) -> Result<Vec<ResolvedStageInputArtifact>> {
    let mut inputs = Vec::new();
    let Some(policies) = explicit_stage_inputs else {
        return Ok(inputs);
    };
    let Some(bindings) = policies.get(&stage_node_id_for_binding(binding)) else {
        return Ok(inputs);
    };
    for stage_input in bindings {
        let source_plan = plans
            .iter()
            .find(|plan| stage_node_id_for_plan(plan) == stage_input.from_stage_node_id)
            .or_else(|| {
                let mut matching_stage_plans = plans
                    .iter()
                    .filter(|plan| plan.stage_id.as_str() == stage_input.from_stage_node_id);
                let first_match = matching_stage_plans.next()?;
                if matching_stage_plans.next().is_some() {
                    return None;
                }
                Some(first_match)
            });
        if let Some(source_plan) = source_plan {
            let artifact = source_plan
                .io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == stage_input.from_output_id)
                .ok_or_else(|| {
                    anyhow!(
                        "stage input binding references missing artifact {} on upstream stage node {}",
                        stage_input.from_output_id,
                        stage_input.from_stage_node_id
                    )
                })?;
            inputs.push(ResolvedStageInputArtifact {
                to_input_id: stage_input.to_input_id.clone(),
                artifact: artifact.clone(),
                source_stage_node_id: stage_input.from_stage_node_id.clone(),
                source_tool_id: source_plan.tool_id.to_string(),
            });
            continue;
        }

        let synthetic_artifact = synthetic_stage_artifacts
            .and_then(|artifacts| artifacts.get(&stage_input.from_stage_node_id))
            .and_then(|artifacts| {
                artifacts
                    .iter()
                    .find(|artifact| artifact.name.as_str() == stage_input.from_output_id)
            })
            .ok_or_else(|| {
                anyhow!(
                    "stage input binding references unknown upstream stage node {}",
                    stage_input.from_stage_node_id
                )
            })?;
        inputs.push(ResolvedStageInputArtifact {
            to_input_id: stage_input.to_input_id.clone(),
            artifact: synthetic_artifact.clone(),
            source_stage_node_id: stage_input.from_stage_node_id.clone(),
            source_tool_id: "planner".to_string(),
        });
    }
    inputs.sort_by(|left, right| {
        left.to_input_id
            .cmp(&right.to_input_id)
            .then_with(|| left.source_stage_node_id.cmp(&right.source_stage_node_id))
            .then_with(|| {
                left.artifact
                    .name
                    .as_str()
                    .cmp(right.artifact.name.as_str())
            })
            .then_with(|| left.artifact.path.cmp(&right.artifact.path))
    });
    Ok(inputs)
}

pub(super) fn has_explicit_input(inputs: &[ResolvedStageInputArtifact], input_id: &str) -> bool {
    inputs.iter().any(|input| input.to_input_id == input_id)
}

fn unique_resolved_input_artifact<'a>(
    inputs: &'a [ResolvedStageInputArtifact],
    input_id: &str,
) -> Result<Option<&'a ResolvedStageInputArtifact>> {
    let mut matches = inputs.iter().filter(|input| input.to_input_id == input_id);
    let first = matches.next();
    let second = matches.next();
    match (first, second) {
        (Some(_), Some(_)) => Err(anyhow!(
            "stage input {} received multiple explicit artifact bindings; provide exactly one binding for singular inputs",
            input_id
        )),
        (Some(input), None) => Ok(Some(input)),
        (None, None) => Ok(None),
        (None, Some(_)) => unreachable!("iterator cannot yield a second item without a first"),
    }
}

pub(super) fn explicit_reference_index_state(
    inputs: &[ResolvedStageInputArtifact],
    input_id: &str,
) -> Result<Option<ReferenceIndexState>> {
    Ok(
        unique_resolved_input_artifact(inputs, input_id)?.map(|input| ReferenceIndexState {
            path: input.artifact.path.clone(),
            tool_id: input.source_tool_id.clone(),
        }),
    )
}

pub(super) fn explicit_reads_input_path(
    inputs: &[ResolvedStageInputArtifact],
    input_id: &str,
) -> Result<Option<PathBuf>> {
    Ok(unique_resolved_input_artifact(inputs, input_id)?.map(|input| input.artifact.path.clone()))
}

pub(super) fn explicit_abundance_table(
    inputs: &[ResolvedStageInputArtifact],
) -> Result<Option<PathBuf>> {
    Ok(unique_resolved_input_artifact(inputs, "abundance_table")?
        .map(|input| input.artifact.path.clone()))
}

pub(super) fn explicit_report_qc_inputs(
    inputs: &[ResolvedStageInputArtifact],
) -> Option<Vec<ArtifactRef>> {
    if inputs.is_empty() {
        return None;
    }
    let mut qc_inputs = inputs
        .iter()
        .filter(|input| input.to_input_id == "qc_artifacts")
        .map(|input| {
            super::report_qc_input_artifact(&input.source_stage_node_id, &input.artifact, None)
        })
        .collect::<Vec<_>>();
    if qc_inputs.is_empty() {
        return None;
    }
    qc_inputs.sort_by(|left, right| {
        left.name
            .as_str()
            .cmp(right.name.as_str())
            .then_with(|| left.path.cmp(&right.path))
    });
    qc_inputs.dedup_by(|left, right| left.name == right.name && left.path == right.path);
    Some(qc_inputs)
}
