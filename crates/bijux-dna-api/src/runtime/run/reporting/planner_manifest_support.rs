use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::request_args::PlanRequest;
use anyhow::Result;
use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use bijux_dna_core::contract::{
    build_plan_manifest, ArtifactRole, ParameterResolutionTraceV1, PlanManifestBuildInputV1,
    PlanManifestDiffV1, PlanManifestV1, PlannerParameterSourceV1, WorkflowInputArtifactV1,
    WorkflowManifestV1, WorkflowStageRequestV1,
};
use sha2::Digest;

pub(crate) fn workflow_manifest_from_request(request: &PlanRequest) -> WorkflowManifestV1 {
    request.workflow_manifest.clone().unwrap_or_else(|| synthesize_workflow_manifest(request))
}

pub(crate) fn plan_manifest_from_request(request: &PlanRequest) -> Result<PlanManifestV1> {
    let workflow_manifest = workflow_manifest_from_request(request);
    let mut effective_parameters_by_step = BTreeMap::new();
    let mut stage_contract_refs = Vec::new();
    for stage_plan in &request.stage_plans {
        let step_id = stage_plan
            .stage_instance_id
            .as_ref()
            .map_or_else(|| stage_plan.stage_id.to_string(), std::string::ToString::to_string);
        effective_parameters_by_step.insert(step_id, stage_plan.effective_params.clone());
        if let Some(contract) = &stage_plan.canonical_contract {
            stage_contract_refs.push((stage_plan.stage_id.to_string(), semantic_hash(contract)?));
        }
    }
    let parameter_traces = if request.parameter_traces.is_empty() {
        infer_parameter_traces(&request.stage_plans)
    } else {
        request.parameter_traces.clone()
    };
    Ok(build_plan_manifest(PlanManifestBuildInputV1 {
        workflow_manifest,
        graph: request.graph.clone(),
        stage_contract_refs,
        effective_parameters_by_step,
        parameter_traces,
        refusal_records: request.planner_refusals.clone(),
        warning_records: request.planner_warnings.clone(),
    })?)
}

pub(crate) fn plan_diff_from_request(
    request: &PlanRequest,
    current: &PlanManifestV1,
) -> Option<PlanManifestDiffV1> {
    request.compare_against.as_ref().map(|baseline| {
        bijux_dna_core::contract::diff_plan_manifests(
            baseline,
            current,
            request.workflow_manifest.as_ref(),
            request.workflow_manifest.as_ref(),
        )
    })
}

fn synthesize_workflow_manifest(request: &PlanRequest) -> WorkflowManifestV1 {
    let mut manifest =
        WorkflowManifestV1::new(infer_domain(&request.graph), request.profile_id.clone());
    manifest.requested_stages = request
        .graph
        .steps()
        .iter()
        .map(|step| WorkflowStageRequestV1 {
            stage_id: step.stage_id.to_string(),
            advisory_only: false,
        })
        .collect();
    manifest.requested_stages.sort_by(|a, b| a.stage_id.cmp(&b.stage_id));
    manifest.inputs = synthetic_inputs_from_graph(&request.graph);
    manifest
}

fn synthetic_inputs_from_graph(
    graph: &bijux_dna_core::contract::ExecutionGraph,
) -> Vec<WorkflowInputArtifactV1> {
    let mut downstream_ids = BTreeSet::new();
    for edge in graph.edges() {
        let Some(step) = graph.step_by_id(edge.to().as_str()) else {
            continue;
        };
        for input in &step.io.inputs {
            downstream_ids.insert(input.name.to_string());
        }
    }
    let mut inputs = graph
        .steps()
        .iter()
        .flat_map(|step| step.io.inputs.iter())
        .filter(|input| {
            !downstream_ids.contains(input.name.as_str()) && input.role != ArtifactRole::Unknown
        })
        .map(|input| WorkflowInputArtifactV1 {
            artifact_id: input.name.to_string(),
            role: input.role,
            path: input.path.clone(),
            layout: None,
            compression: None,
            format_id: None,
        })
        .collect::<Vec<_>>();
    inputs.sort_by(|a, b| a.artifact_id.cmp(&b.artifact_id));
    inputs.dedup_by(|a, b| a.artifact_id == b.artifact_id);
    inputs
}

fn infer_domain(graph: &bijux_dna_core::contract::ExecutionGraph) -> String {
    graph
        .steps()
        .first()
        .map(|step| step.stage_id.as_str().split('.').next().unwrap_or("core").to_string())
        .unwrap_or_else(|| "core".to_string())
}

fn infer_parameter_traces(
    stage_plans: &[bijux_dna_stage_contract::StagePlanV1],
) -> Vec<ParameterResolutionTraceV1> {
    stage_plans
        .iter()
        .flat_map(|stage_plan| {
            let step_id = stage_plan
                .stage_instance_id
                .as_ref()
                .map_or_else(|| stage_plan.stage_id.to_string(), std::string::ToString::to_string);
            let serde_json::Value::Object(map) = &stage_plan.effective_params else {
                return Vec::new();
            };
            map.iter()
                .map(|(parameter, value)| ParameterResolutionTraceV1 {
                    step_id: step_id.clone(),
                    stage_id: stage_plan.stage_id.to_string(),
                    parameter: parameter.clone(),
                    source: if stage_plan.params.get(parameter) == Some(value) {
                        PlannerParameterSourceV1::PlannerInferred
                    } else {
                        PlannerParameterSourceV1::BackendConstraint
                    },
                    resolved_value: value.clone(),
                    detail: "inferred from planner stage contract because no explicit trace was provided"
                        .to_string(),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn semantic_hash<T: serde::Serialize>(value: &T) -> Result<String> {
    let bytes = to_canonical_json_bytes(value)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    Ok(hex)
}
