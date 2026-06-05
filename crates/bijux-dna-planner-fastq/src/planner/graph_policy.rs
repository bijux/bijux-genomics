#![allow(clippy::uninlined_format_args, clippy::unnecessary_wraps, clippy::wildcard_imports)]

use super::*;

pub(super) fn validate_select_stage_nodes(
    pipeline_spec: &PipelineSpec,
    stage_bindings: &[FastqStageBinding],
) -> Result<()> {
    if !pipeline_spec.declares_graph_topology() {
        return Ok(());
    }
    let binding_by_node_id = stage_bindings
        .iter()
        .map(|binding| (binding_node_id(binding), binding))
        .collect::<std::collections::BTreeMap<_, _>>();
    for node in pipeline_spec
        .ordered_nodes()
        .into_iter()
        .filter(|node| node.stage_id == crate::STAGE_SELECT_STAGE_TOOL.as_str())
    {
        let node_id =
            PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref());
        if node.stage_instance_id.is_none() {
            return Err(anyhow!(
                "selection node {} must set stage_instance_id to remain graph-addressable",
                node.stage_id
            ));
        }
        let incoming =
            pipeline_spec.edges.iter().filter(|edge| edge.to == node_id).collect::<Vec<_>>();
        if incoming.len() < 2 {
            return Err(anyhow!(
                "selection node {} requires at least two incoming candidate edges",
                node_id
            ));
        }
        let mut source_stage_id = None::<String>;
        let mut candidate_input_ids = std::collections::BTreeSet::new();
        for edge in &incoming {
            let (Some(_from_output_id), Some(to_input_id)) =
                (&edge.from_output_id, &edge.to_input_id)
            else {
                return Err(anyhow!(
                    "selection node {} requires artifact-bound incoming edges",
                    node_id
                ));
            };
            if !candidate_input_ids.insert(to_input_id.clone()) {
                return Err(anyhow!(
                    "selection node {} requires unique candidate input ids per incoming edge",
                    node_id
                ));
            }
            let Some(source_binding) = binding_by_node_id.get(&edge.from).copied() else {
                return Err(anyhow!(
                    "selection node {} references unknown source node {}",
                    node_id,
                    edge.from
                ));
            };
            if let Some(expected_stage_id) = source_stage_id.as_ref() {
                if expected_stage_id != &source_binding.stage_id {
                    return Err(anyhow!(
                        "selection node {} must join candidate tools from one stage family",
                        node_id
                    ));
                }
            } else {
                source_stage_id = Some(source_binding.stage_id.clone());
            }
        }
        for edge in pipeline_spec.edges.iter().filter(|edge| edge.from == node_id) {
            let (Some(_from_output_id), Some(_to_input_id)) =
                (&edge.from_output_id, &edge.to_input_id)
            else {
                return Err(anyhow!(
                    "selection node {} requires artifact-bound outgoing rejoin edges",
                    node_id
                ));
            };
        }
    }
    Ok(())
}

pub(super) fn planner_owned_graph_stage(stage_id: &str) -> bool {
    stage_id == crate::STAGE_SELECT_STAGE_TOOL.as_str()
}

pub(super) fn synthetic_stage_artifact_policy(
    pipeline_spec: &PipelineSpec,
    root_out_dir: &std::path::Path,
) -> Result<crate::compose::SyntheticStageArtifactPolicy> {
    let mut artifacts = crate::compose::SyntheticStageArtifactPolicy::new();
    if !pipeline_spec.declares_graph_topology() {
        return Ok(artifacts);
    }
    for node in pipeline_spec
        .ordered_nodes()
        .into_iter()
        .filter(|node| node.stage_id == crate::STAGE_SELECT_STAGE_TOOL.as_str())
    {
        let node_id =
            PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref());
        let artifact_ids = pipeline_spec
            .edges
            .iter()
            .filter(|edge| edge.from == node_id)
            .filter_map(|edge| edge.from_output_id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let select_out_dir =
            root_out_dir.join(node_id.trim_start_matches(STAGE_PREFIX)).join("selected");
        artifacts.insert(
            node_id,
            artifact_ids
                .into_iter()
                .map(|artifact_id| crate::compose::SyntheticStageArtifact {
                    artifact: ArtifactRef::required(
                        ArtifactId::new(artifact_id.clone()),
                        select_out_dir.join(selection_artifact_file_name(&artifact_id)),
                        inferred_selection_artifact_role(&artifact_id),
                    ),
                    source_tool_id: "planner".to_string(),
                })
                .collect(),
        );
    }
    Ok(artifacts)
}

pub(super) fn validate_reference_index_bindings(
    bindings: &[FastqStageBinding],
    pipeline_spec: &PipelineSpec,
) -> Result<()> {
    let explicit_stage_inputs = stage_artifact_input_policy(pipeline_spec);
    let binding_by_node_id = bindings
        .iter()
        .map(|binding| {
            let node_id = binding_node_id(binding);
            (node_id, binding)
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let dependency_policy = stage_dependency_policy(pipeline_spec);
    let mut current_index_backend: Option<&str> = None;
    for binding in bindings {
        match binding.stage_id.as_str() {
            "fastq.index_reference" => {
                current_index_backend = Some(binding.tool.tool_id.as_str());
            }
            "fastq.deplete_host" | "fastq.deplete_reference_contaminants" => {
                let explicit_backend = explicit_reference_index_binding(
                    binding,
                    &explicit_stage_inputs,
                    &binding_by_node_id,
                )?
                .map(|upstream_binding| upstream_binding.tool.tool_id.as_str());
                let dependency_backend = dependency_reference_index_binding(
                    binding,
                    &dependency_policy,
                    &binding_by_node_id,
                )?
                .map(|upstream_binding| upstream_binding.tool.tool_id.as_str());
                let Some(index_backend) =
                    explicit_backend.or(dependency_backend).or(current_index_backend)
                else {
                    continue;
                };
                let depletion_tool_id =
                    bijux_dna_core::ids::ToolId::new(binding.tool.tool_id.as_str().to_string());
                let index_backend_id = bijux_dna_core::ids::ToolId::new(index_backend.to_string());
                if bijux_dna_domain_fastq::is_reference_index_backend_compatible(
                    &depletion_tool_id,
                    &index_backend_id,
                ) {
                    continue;
                }
                let compatible_backends =
                    bijux_dna_domain_fastq::reference_index_backends_for_tool(&depletion_tool_id);
                return Err(anyhow!(
                    "{} requires one of [{}] as reference index backend, but upstream fastq.index_reference selected {}",
                    binding.stage_id,
                    compatible_backends
                        .iter()
                        .map(|tool_id| tool_id.as_str().to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    index_backend
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn binding_node_id(binding: &FastqStageBinding) -> String {
    binding.stage_instance_id.clone().unwrap_or_else(|| binding.stage_id.clone())
}

pub(super) fn stage_dependency_policy(
    pipeline_spec: &PipelineSpec,
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut dependencies = std::collections::BTreeMap::<String, Vec<String>>::new();
    if !pipeline_spec.declares_graph_topology() {
        return dependencies;
    }
    for node in pipeline_spec.ordered_nodes() {
        let node_id =
            PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref());
        dependencies.entry(node_id).or_default();
    }
    for edge in &pipeline_spec.edges {
        dependencies.entry(edge.to.clone()).or_default().push(edge.from.clone());
    }
    dependencies
}

fn explicit_reference_index_binding<'a>(
    binding: &FastqStageBinding,
    explicit_stage_inputs: &'a crate::compose::StageArtifactInputPolicy,
    binding_by_node_id: &'a std::collections::BTreeMap<String, &'a FastqStageBinding>,
) -> Result<Option<&'a FastqStageBinding>> {
    Ok(explicit_stage_inputs
        .get(&binding_node_id(binding))
        .and_then(|inputs| inputs.iter().find(|input| input.to_input_id == "reference_index"))
        .and_then(|input| binding_by_node_id.get(&input.from_stage_node_id).copied()))
}

fn dependency_reference_index_binding<'a>(
    binding: &FastqStageBinding,
    dependency_policy: &'a std::collections::BTreeMap<String, Vec<String>>,
    binding_by_node_id: &'a std::collections::BTreeMap<String, &'a FastqStageBinding>,
) -> Result<Option<&'a FastqStageBinding>> {
    let node_id = binding_node_id(binding);
    let Some(upstream_nodes) = dependency_policy.get(&node_id) else {
        return Ok(None);
    };
    let mut upstream_indices = upstream_nodes
        .iter()
        .filter_map(|upstream_node| binding_by_node_id.get(upstream_node).copied())
        .filter(|upstream| upstream.stage_id == "fastq.index_reference")
        .collect::<Vec<_>>();
    upstream_indices.sort_by_key(|left| binding_node_id(left));
    upstream_indices.dedup_by(|left, right| binding_node_id(left) == binding_node_id(right));
    match upstream_indices.len() {
        0 => Ok(None),
        1 => Ok(upstream_indices.into_iter().next()),
        _ => Err(anyhow!(
            "{} depends on multiple fastq.index_reference nodes; add an explicit reference_index artifact binding",
            binding.stage_id
        )),
    }
}

pub(super) fn execution_edges_for_stage_plans(
    pipeline_spec: &PipelineSpec,
    plans: &[StagePlanV1],
    synthetic_step_nodes: &std::collections::BTreeMap<String, StepId>,
) -> Result<Vec<ExecutionEdge>> {
    let mut plan_nodes = std::collections::BTreeMap::new();
    let mut stage_counts = std::collections::BTreeMap::new();
    for plan in plans {
        *stage_counts.entry(plan.stage_id.as_str().to_string()).or_insert(0usize) += 1;
    }
    for plan in plans {
        let node_id = plan
            .stage_instance_id
            .as_ref()
            .map_or_else(|| plan.stage_id.as_str().to_string(), ToString::to_string);
        let step_id = StepId::new(node_id.clone());
        plan_nodes.insert(node_id, step_id.clone());
        if stage_counts.get(plan.stage_id.as_str()).copied() == Some(1) {
            plan_nodes.insert(plan.stage_id.as_str().to_string(), step_id);
        }
    }
    plan_nodes.extend(
        synthetic_step_nodes.iter().map(|(node_id, step_id)| (node_id.clone(), step_id.clone())),
    );
    for node in pipeline_spec.ordered_nodes() {
        let node_id =
            PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref());
        if !plan_nodes.contains_key(&node_id) {
            return Err(anyhow!(
                "pipeline graph references stage node {} but planner did not produce a matching step",
                node_id
            ));
        }
    }

    let mut edges = pipeline_spec
        .edges
        .iter()
        .map(|edge| execution_edge_from_pipeline_edge(edge, &plan_nodes))
        .collect::<Result<Vec<_>>>()?;
    edges.extend(derived_lineage_execution_edges(plans, &plan_nodes));
    edges.sort_by(|left, right| {
        left.from()
            .as_str()
            .cmp(right.from().as_str())
            .then_with(|| left.to().as_str().cmp(right.to().as_str()))
            .then_with(|| left.from_output_id().cmp(&right.from_output_id()))
            .then_with(|| left.to_input_id().cmp(&right.to_input_id()))
    });
    let artifact_bound_pairs = edges
        .iter()
        .filter(|edge| edge.from_output_id().is_some() && edge.to_input_id().is_some())
        .map(|edge| (edge.from().as_str().to_string(), edge.to().as_str().to_string()))
        .collect::<std::collections::BTreeSet<_>>();
    edges.retain(|edge| {
        edge.from_output_id().is_some()
            || !artifact_bound_pairs
                .contains(&(edge.from().as_str().to_string(), edge.to().as_str().to_string()))
    });
    edges.dedup_by(|left, right| {
        left.from() == right.from()
            && left.to() == right.to()
            && left.from_output_id() == right.from_output_id()
            && left.to_input_id() == right.to_input_id()
    });
    Ok(edges)
}

fn derived_lineage_execution_edges(
    plans: &[StagePlanV1],
    plan_nodes: &std::collections::BTreeMap<String, StepId>,
) -> Vec<ExecutionEdge> {
    let mut edges = Vec::new();
    for (to_idx, to_plan) in plans.iter().enumerate() {
        let Some(to_step_id) = plan_nodes.get(
            to_plan
                .stage_instance_id
                .as_ref()
                .map_or_else(|| to_plan.stage_id.as_str(), |step_id| step_id.as_str()),
        ) else {
            continue;
        };
        for input in &to_plan.io.inputs {
            let Some((from_plan, output)) = plans[..to_idx].iter().rev().find_map(|candidate| {
                candidate
                    .io
                    .outputs
                    .iter()
                    .find(|output| output.name == input.name && output.path == input.path)
                    .map(|output| (candidate, output))
            }) else {
                continue;
            };
            let Some(from_step_id) = plan_nodes.get(
                from_plan
                    .stage_instance_id
                    .as_ref()
                    .map_or_else(|| from_plan.stage_id.as_str(), |step_id| step_id.as_str()),
            ) else {
                continue;
            };
            edges.push(ExecutionEdge::with_artifact_binding(
                from_step_id.clone(),
                to_step_id.clone(),
                ArtifactId::new(output.name.as_str().to_string()),
                ArtifactId::new(input.name.as_str().to_string()),
            ));
        }
    }
    edges
}

pub(super) fn stage_artifact_input_policy(
    pipeline_spec: &PipelineSpec,
) -> crate::compose::StageArtifactInputPolicy {
    let mut policies = crate::compose::StageArtifactInputPolicy::new();
    if !pipeline_spec.declares_graph_topology() {
        return policies;
    }
    for edge in &pipeline_spec.edges {
        let (Some(from_output_id), Some(to_input_id)) = (&edge.from_output_id, &edge.to_input_id)
        else {
            continue;
        };
        policies.entry(edge.to.clone()).or_default().push(
            crate::compose::StageArtifactInputBinding {
                from_stage_node_id: edge.from.clone(),
                from_output_id: from_output_id.clone(),
                to_input_id: to_input_id.clone(),
            },
        );
    }
    policies
}

fn execution_edge_from_pipeline_edge(
    edge: &PipelineEdgeSpec,
    plan_nodes: &std::collections::BTreeMap<String, StepId>,
) -> Result<ExecutionEdge> {
    let from = plan_nodes.get(&edge.from).cloned().ok_or_else(|| {
        anyhow!("pipeline graph edge source {} does not resolve to a planned step", edge.from)
    })?;
    let to = plan_nodes.get(&edge.to).cloned().ok_or_else(|| {
        anyhow!("pipeline graph edge target {} does not resolve to a planned step", edge.to)
    })?;
    match (&edge.from_output_id, &edge.to_input_id) {
        (Some(from_output_id), Some(to_input_id)) => Ok(ExecutionEdge::with_artifact_binding(
            from,
            to,
            ArtifactId::new(from_output_id.clone()),
            ArtifactId::new(to_input_id.clone()),
        )),
        (None, None) => Ok(ExecutionEdge::new(from, to)),
        _ => Err(anyhow!(
            "pipeline graph edge {} -> {} must set both from_output_id and to_input_id together",
            edge.from,
            edge.to
        )),
    }
}

pub(super) fn ensure_unique_stage_binding_nodes(bindings: &[FastqStageBinding]) -> Result<()> {
    let mut seen_nodes = std::collections::BTreeSet::new();
    for binding in bindings {
        let node_id = binding.stage_instance_id.as_deref().map_or_else(
            || format!("{}.tool.{}", binding.stage_id, binding.tool.tool_id.as_str()),
            str::to_string,
        );
        if !seen_nodes.insert(node_id.clone()) {
            return Err(anyhow!(
                "duplicate FASTQ stage node binding {}; repeated stage/tool bindings must set distinct stage_instance_id values",
                node_id
            ));
        }
    }
    Ok(())
}

pub(crate) fn stage_status(stage_id: &str) -> Option<String> {
    let stage_id = bijux_dna_core::ids::StageId::try_from(stage_id).ok()?;
    bijux_dna_domain_fastq::execution_support_for_stage(&stage_id).map(|support| {
        match support.execution_status {
            bijux_dna_domain_fastq::ExecutionStatus::Closed => "supported",
            bijux_dna_domain_fastq::ExecutionStatus::DeclaredOnly => "planned",
        }
        .to_string()
    })
}

pub(super) fn enforce_stage_status(stage_id: &str, allow_planned: bool) -> Result<()> {
    match stage_status(stage_id).as_deref() {
        Some("supported") | None => Ok(()),
        Some("planned" | "out_of_scope") if allow_planned => Ok(()),
        Some("planned" | "out_of_scope") => Err(anyhow!(
            "stage {stage_id} is not active in current scope; re-run with --allow-planned to override"
        )),
        Some(other) => Err(anyhow!("stage {stage_id} has unknown status {other}")),
    }
}
