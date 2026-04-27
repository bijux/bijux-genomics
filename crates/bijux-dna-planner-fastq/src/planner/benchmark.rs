#![allow(clippy::uninlined_format_args, clippy::wildcard_imports)]

use super::*;

pub(super) fn project_benchmark_stage_params_for_tool(
    stage_id: &StageId,
    tool_id: &bijux_dna_core::ids::ToolId,
    params: Option<&FastqStageParameters>,
) -> Option<FastqStageParameters> {
    match (stage_id.as_str(), params) {
        ("fastq.correct_errors", Some(FastqStageParameters::CorrectErrors(params))) => {
            Some(FastqStageParameters::CorrectErrors(project_correct_errors_params_for_tool(
                tool_id.as_str(),
                params,
            )))
        }
        (_, Some(params)) => Some(params.clone()),
        (_, None) => None,
    }
}

fn project_correct_errors_params_for_tool(
    tool_id: &str,
    params: &CorrectErrorsStageParams,
) -> CorrectErrorsStageParams {
    crate::tool_adapters::stages::transform::correct_errors::project_correct_options_for_tool(
        tool_id, params,
    )
}

pub(super) fn comparison_command_for_stage(
    stage_id: &StageId,
    comparison_artifact_ids: &[String],
) -> Result<Vec<String>> {
    let mut command = vec![
        "stage-tool-compare".to_string(),
        "--stage".to_string(),
        stage_id.as_str().to_string(),
    ];
    if let Some(scenario) = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(stage_id)
        .into_iter()
        .map(|scenario| scenario.scenario_id)
        .next()
    {
        command.push("--scenario".to_string());
        command.push(scenario);
    }
    if let Some(contract_hash) = bijux_dna_domain_fastq::stage_contract_hash(stage_id.as_str()) {
        command.push("--stage-contract-hash".to_string());
        command.push(contract_hash.map_err(|err| {
            anyhow!(
                "compute stage contract hash for benchmark compare {}: {err}",
                stage_id.as_str()
            )
        })?);
    }
    for artifact_id in bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(stage_id) {
        command.push("--comparison-input".to_string());
        command.push(artifact_id.to_string());
    }
    for artifact_id in comparison_artifact_ids {
        command.push("--comparison-artifact".to_string());
        command.push(artifact_id.clone());
    }
    Ok(command)
}

pub(super) fn comparison_artifact_file_name(artifact_id: &str) -> String {
    let stem = artifact_id.strip_suffix("_json").unwrap_or(artifact_id);
    format!("{stem}.json")
}

pub(super) fn selection_artifact_file_name(artifact_id: &str) -> String {
    if artifact_id.ends_with("_json") || artifact_id.ends_with("_manifest") {
        let stem = artifact_id
            .strip_suffix("_json")
            .or_else(|| artifact_id.strip_suffix("_manifest"))
            .unwrap_or(artifact_id);
        return format!("{stem}.json");
    }
    if artifact_id.ends_with("_html") {
        let stem = artifact_id.strip_suffix("_html").unwrap_or(artifact_id);
        return format!("{stem}.html");
    }
    if artifact_id.contains("reads") || artifact_id.ends_with("_r1") || artifact_id.ends_with("_r2")
    {
        return format!("{artifact_id}.fastq.gz");
    }
    if artifact_id.contains("index") {
        return format!("{artifact_id}.idx");
    }
    if artifact_id.contains("table") {
        return format!("{artifact_id}.tsv");
    }
    artifact_id.to_string()
}

pub(super) fn inferred_selection_artifact_role(artifact_id: &str) -> ArtifactRole {
    if artifact_id == "report_html" {
        return ArtifactRole::ReportHtml;
    }
    if artifact_id.ends_with("_json") || artifact_id.ends_with("_manifest") {
        return ArtifactRole::SummaryJson;
    }
    if artifact_id.contains("report") {
        return ArtifactRole::ReportJson;
    }
    if artifact_id.contains("index") {
        return ArtifactRole::Index;
    }
    if artifact_id.contains("table") {
        return ArtifactRole::SummaryTsv;
    }
    if artifact_id.contains("trimmed_reads") {
        return ArtifactRole::TrimmedReads;
    }
    if artifact_id.contains("reads") || artifact_id.ends_with("_r1") || artifact_id.ends_with("_r2")
    {
        return ArtifactRole::Reads;
    }
    ArtifactRole::Unknown
}

fn selection_command_for_stage(
    stage_id: &StageId,
    output_artifact_ids: &[String],
    objective: bijux_dna_core::contract::Objective,
) -> Vec<String> {
    let mut command = vec![
        "stage-tool-select".to_string(),
        "--stage".to_string(),
        stage_id.as_str().to_string(),
        "--objective".to_string(),
        objective.as_str().to_string(),
    ];
    for artifact_id in output_artifact_ids {
        command.push("--selected-artifact".to_string());
        command.push(artifact_id.clone());
    }
    command
}

pub(super) fn benchmark_compare_steps_for_toolsets(
    config: &FastqPlanConfig,
    plans: &[StagePlanV1],
) -> Result<(Vec<ExecutionStep>, Vec<ExecutionEdge>)> {
    if config.stage_toolsets.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut steps = Vec::new();
    let mut edges = Vec::new();
    for toolset in config.stage_toolsets.iter().filter(|binding| binding.tools.len() > 1) {
        let stage_id = StageId::new(toolset.stage_id.clone());
        let comparison_artifact_ids =
            bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&stage_id);
        if comparison_artifact_ids.is_empty() {
            continue;
        }
        let comparison_input_artifact_ids =
            bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&stage_id);
        let stage_node_id =
            PipelineSpec::stage_node_id(&toolset.stage_id, toolset.stage_instance_id.as_deref());
        let mut plans_by_context = std::collections::BTreeMap::<String, Vec<&StagePlanV1>>::new();
        for plan in plans.iter().filter(|plan| plan_originates_from_toolset(plan, toolset)) {
            let context_key = compare_context_key_for_plan(plan, &stage_node_id);
            plans_by_context.entry(context_key).or_default().push(plan);
        }

        for (context_key, context_plans) in plans_by_context {
            let distinct_tools = context_plans
                .iter()
                .map(|plan| plan.tool_id.as_str())
                .collect::<std::collections::BTreeSet<_>>();
            if distinct_tools.len() < 2 {
                continue;
            }
            let compare_step_id = compare_step_id_for_context(&stage_node_id, &context_key);
            let compare_out_dir =
                compare_out_dir_for_context(&config.out_dir, &stage_node_id, &context_key);
            let comparison_command =
                comparison_command_for_stage(&stage_id, &comparison_artifact_ids)?;
            let comparison_outputs = comparison_artifact_ids
                .iter()
                .map(|artifact_id| {
                    ArtifactRef::required(
                        ArtifactId::new(artifact_id.clone()),
                        compare_out_dir.join(comparison_artifact_file_name(artifact_id)),
                        ArtifactRole::SummaryJson,
                    )
                })
                .collect::<Vec<_>>();
            let mut comparison_inputs = Vec::new();
            for plan in &context_plans {
                for output in &plan.io.outputs {
                    if !comparison_input_artifact_ids.is_empty()
                        && !comparison_input_artifact_ids
                            .iter()
                            .any(|artifact_id| *artifact_id == output.name.as_str())
                    {
                        continue;
                    }
                    comparison_inputs.push(ArtifactRef::required(
                        ArtifactId::new(format!(
                            "{}__{}",
                            plan.tool_id.as_str(),
                            output.name.as_str()
                        )),
                        output.path.clone(),
                        output.role,
                    ));
                }
                edges.push(ExecutionEdge::new(step_id_for_plan(plan), compare_step_id.clone()));
            }
            comparison_inputs.sort_by(|left, right| {
                left.name.as_str().cmp(right.name.as_str()).then_with(|| left.path.cmp(&right.path))
            });
            steps.push(ExecutionStep {
                step_id: compare_step_id,
                stage_id: crate::STAGE_COMPARE_STAGE_TOOLS,
                command: CommandSpecV1 { template: comparison_command },
                image: ContainerImageRefV1 { image: "bijux-dna-compare".to_string(), digest: None },
                resources: ToolConstraints::default(),
                io: StageIO { inputs: comparison_inputs, outputs: comparison_outputs },
                out_dir: compare_out_dir,
                aux_images: BTreeMap::new(),
                expected_artifact_ids: comparison_artifact_ids
                    .iter()
                    .map(|artifact_id| ArtifactId::new(artifact_id.clone()))
                    .collect(),
                metrics_schema_ids: Vec::new(),
            });
        }
    }

    Ok((steps, edges))
}

type SelectStepPlan =
    (Vec<ExecutionStep>, Vec<ExecutionEdge>, std::collections::BTreeMap<String, StepId>);
type PlanByNodeId<'a> = std::collections::BTreeMap<String, &'a StagePlanV1>;

pub(super) fn benchmark_select_steps_for_pipeline(
    config: &FastqPlanConfig,
    pipeline_spec: &PipelineSpec,
    plans: &[StagePlanV1],
) -> Result<SelectStepPlan> {
    if !pipeline_spec.declares_graph_topology() {
        return Ok((Vec::new(), Vec::new(), std::collections::BTreeMap::new()));
    }

    let plan_by_node_id = plan_by_node_id(plans);
    let mut steps = Vec::new();
    let edges = Vec::new();
    let mut step_nodes = std::collections::BTreeMap::new();
    for node in pipeline_spec
        .ordered_nodes()
        .into_iter()
        .filter(|node| node.stage_id == crate::STAGE_SELECT_STAGE_TOOL.as_str())
    {
        let node_id =
            PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref());
        let incoming_edges =
            pipeline_spec.edges.iter().filter(|edge| edge.to == node_id).collect::<Vec<_>>();
        if incoming_edges.is_empty() {
            continue;
        }
        let source_stage_id =
            selection_source_stage_id(&node_id, &incoming_edges, &plan_by_node_id)?;
        let select_out_dir =
            config.out_dir.join(node_id.trim_start_matches(STAGE_PREFIX)).join("selected");
        let output_artifact_ids = selection_output_artifact_ids(pipeline_spec, &node_id);
        let inputs = selection_inputs(&node_id, &incoming_edges, &plan_by_node_id)?;
        let outputs = selection_outputs(&output_artifact_ids, &select_out_dir);
        let step_id = StepId::new(node_id.clone());
        steps.push(selection_step(
            step_id.clone(),
            &source_stage_id,
            output_artifact_ids,
            inputs,
            outputs,
            select_out_dir,
            config.selection_objective,
        ));
        step_nodes.insert(node_id, step_id);
    }

    Ok((steps, edges, step_nodes))
}

fn plan_by_node_id(plans: &[StagePlanV1]) -> PlanByNodeId<'_> {
    plans.iter().map(|plan| (step_id_for_plan(plan).as_str().to_string(), plan)).collect()
}

fn selection_source_stage_id(
    node_id: &str,
    incoming_edges: &[&PipelineEdgeSpec],
    plan_by_node_id: &PlanByNodeId<'_>,
) -> Result<StageId> {
    let source_plans = incoming_edges
        .iter()
        .map(|edge| {
            plan_by_node_id.get(&edge.from).copied().ok_or_else(|| {
                anyhow!("selection node {} references unknown upstream step {}", node_id, edge.from)
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let source_stage_id = StageId::new(source_plans[0].stage_id.as_str().to_string());
    if source_plans.iter().any(|plan| plan.stage_id.as_str() != source_stage_id.as_str()) {
        return Err(anyhow!(
            "selection node {} must join candidates from one stage family",
            node_id
        ));
    }
    Ok(source_stage_id)
}

fn selection_output_artifact_ids(pipeline_spec: &PipelineSpec, node_id: &str) -> Vec<String> {
    pipeline_spec
        .edges
        .iter()
        .filter(|edge| edge.from == node_id)
        .filter_map(|edge| edge.from_output_id.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn selection_outputs(
    output_artifact_ids: &[String],
    select_out_dir: &std::path::Path,
) -> Vec<ArtifactRef> {
    output_artifact_ids
        .iter()
        .map(|artifact_id| {
            ArtifactRef::required(
                ArtifactId::new(artifact_id.clone()),
                select_out_dir.join(selection_artifact_file_name(artifact_id)),
                inferred_selection_artifact_role(artifact_id),
            )
        })
        .collect()
}

fn selection_inputs(
    node_id: &str,
    incoming_edges: &[&PipelineEdgeSpec],
    plan_by_node_id: &PlanByNodeId<'_>,
) -> Result<Vec<ArtifactRef>> {
    let mut inputs = incoming_edges
        .iter()
        .map(|edge| selection_input_for_edge(node_id, edge, plan_by_node_id))
        .collect::<Result<Vec<_>>>()?;
    inputs.sort_by(|left, right| {
        left.name.as_str().cmp(right.name.as_str()).then_with(|| left.path.cmp(&right.path))
    });
    inputs.dedup_by(|left, right| left.name == right.name && left.path == right.path);
    Ok(inputs)
}

fn selection_input_for_edge(
    node_id: &str,
    edge: &PipelineEdgeSpec,
    plan_by_node_id: &PlanByNodeId<'_>,
) -> Result<ArtifactRef> {
    let source_plan = plan_by_node_id.get(&edge.from).copied().ok_or_else(|| {
        anyhow!("selection node {} references unresolved source plan {}", node_id, edge.from)
    })?;
    let source_output_id = edge.from_output_id.as_ref().ok_or_else(|| {
        anyhow!("selection node {} requires bound source output ids on incoming edges", node_id)
    })?;
    let source_output = source_plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == source_output_id)
        .ok_or_else(|| {
            anyhow!(
                "selection node {} references missing source artifact {} on {}",
                node_id,
                source_output_id,
                edge.from
            )
        })?;
    Ok(ArtifactRef::required(
        ArtifactId::new(edge.to_input_id.clone().ok_or_else(|| {
            anyhow!(
                "selection node {} requires bound destination input ids on incoming edges",
                node_id
            )
        })?),
        source_output.path.clone(),
        source_output.role,
    ))
}

fn selection_step(
    step_id: StepId,
    source_stage_id: &StageId,
    output_artifact_ids: Vec<String>,
    inputs: Vec<ArtifactRef>,
    outputs: Vec<ArtifactRef>,
    select_out_dir: std::path::PathBuf,
    objective: bijux_dna_core::contract::Objective,
) -> ExecutionStep {
    ExecutionStep {
        step_id,
        stage_id: crate::STAGE_SELECT_STAGE_TOOL,
        command: CommandSpecV1 {
            template: selection_command_for_stage(source_stage_id, &output_artifact_ids, objective),
        },
        image: ContainerImageRefV1 { image: "bijux-dna-select".to_string(), digest: None },
        resources: ToolConstraints::default(),
        io: StageIO { inputs, outputs },
        out_dir: select_out_dir,
        aux_images: BTreeMap::new(),
        expected_artifact_ids: output_artifact_ids.into_iter().map(ArtifactId::new).collect(),
        metrics_schema_ids: Vec::new(),
    }
}

fn plan_originates_from_toolset(plan: &StagePlanV1, toolset: &FastqStageToolsetBinding) -> bool {
    if plan.stage_id.as_str() != toolset.stage_id {
        return false;
    }
    let plan_stage_instance_id = plan.stage_instance_id.as_ref().map(ToString::to_string);
    source_toolset_for_expanded_selection(
        std::slice::from_ref(toolset),
        plan.stage_id.as_str(),
        plan_stage_instance_id.as_deref(),
    )
    .is_some()
}

fn compare_context_key_for_plan(plan: &StagePlanV1, stage_node_id: &str) -> String {
    let Some(assignments) = expanded_route_assignments(
        plan.stage_instance_id.as_ref().map(bijux_dna_core::contract::StepId::as_str),
    ) else {
        return String::new();
    };
    assignments
        .into_iter()
        .filter(|(node_id, _)| node_id != stage_node_id)
        .map(|(node_id, tool_id)| format!("{node_id}={tool_id}"))
        .collect::<Vec<_>>()
        .join("__")
}

fn expanded_route_assignments(
    stage_instance_id: Option<&str>,
) -> Option<std::collections::BTreeMap<String, String>> {
    let stage_instance_id = stage_instance_id?;
    let (_, route_and_tool) = stage_instance_id.split_once(".route.")?;
    let (route_key, _) = route_and_tool.rsplit_once(".tool.")?;
    let mut assignments = std::collections::BTreeMap::new();
    for assignment in route_key.split("__") {
        let (node_id, tool_id) = assignment.split_once('=')?;
        assignments.insert(node_id.to_string(), tool_id.to_string());
    }
    Some(assignments)
}

fn compare_step_id_for_context(stage_node_id: &str, context_key: &str) -> StepId {
    if context_key.is_empty() {
        return StepId::new(format!("{stage_node_id}.compare"));
    }
    StepId::new(format!("{stage_node_id}.compare.route.{context_key}"))
}

fn compare_out_dir_for_context(
    root_out_dir: &std::path::Path,
    stage_node_id: &str,
    context_key: &str,
) -> std::path::PathBuf {
    let compare_dir =
        root_out_dir.join(stage_node_id.trim_start_matches(STAGE_PREFIX)).join("compare");
    if context_key.is_empty() {
        compare_dir
    } else {
        compare_dir.join(context_key)
    }
}

fn step_id_for_plan(plan: &StagePlanV1) -> StepId {
    plan.stage_instance_id
        .clone()
        .unwrap_or_else(|| StepId::new(plan.stage_id.as_str().to_string()))
}
