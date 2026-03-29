use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::contract::{
    ArtifactRef, ArtifactRole, ExecutionEdge, ExecutionGraph, ExecutionStep, StageIO,
    ToolConstraints,
};
use bijux_dna_core::contract::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec};
use bijux_dna_core::prelude::{
    ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId, ToolExecutionSpecV1,
};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_domain_fastq::preprocess_pipeline_graph_for_stage_order;
use bijux_dna_domain_fastq::{
    stages::ids::{STAGE_DEPLETE_HOST, STAGE_DEPLETE_REFERENCE_CONTAMINANTS},
    FastqPipelineMode, STAGE_CLUSTER_OTUS, STAGE_CORRECT_ERRORS, STAGE_DEPLETE_RRNA,
    STAGE_DETECT_ADAPTERS, STAGE_EXTRACT_UMIS, STAGE_FILTER_LOW_COMPLEXITY, STAGE_FILTER_READS,
    STAGE_INFER_ASVS, STAGE_MERGE_PAIRS, STAGE_NORMALIZE_ABUNDANCE, STAGE_NORMALIZE_PRIMERS,
    STAGE_PREFIX, STAGE_PROFILE_READS, STAGE_REMOVE_CHIMERAS, STAGE_REMOVE_DUPLICATES,
    STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY, STAGE_TRIM_READS, STAGE_TRIM_TERMINAL_DAMAGE,
    STAGE_VALIDATE_READS,
};
use bijux_dna_pipelines::STAGE_CORE_PREPARE_REFERENCE;
use bijux_dna_stage_contract::{
    default_edges_for_stages, PlanDecisionReason, PlanReasonKind, StagePlanV1,
};

use crate::{
    default_pipeline_spec, plan_compose, BenchResultsRepository, DefaultPipelineOptions,
    PLANNER_VERSION, STAGE_PREPROCESS_SUMMARY, TOOL_SEQKIT, required_id_catalog,
};

mod support;
mod types;

pub(crate) use support::{apply_layout_branching, estimate_mean_q};
pub use types::*;

pub struct FastqPlanner;

const DEFAULT_MAX_ROUTE_SPECIFIC_PIPELINES: usize = 4096;

impl FastqPlanner {
    /// # Errors
    /// Returns an error if planning fails or the plan lint fails.
    pub fn plan(config: &FastqPlanConfig) -> Result<ExecutionGraph> {
        let (pipeline_spec, stage_bindings) = normalize_stage_bindings(config)?;
        validate_select_stage_nodes(&pipeline_spec, &stage_bindings)?;
        validate_reference_index_bindings(&stage_bindings, &pipeline_spec)?;
        for binding in &stage_bindings {
            enforce_stage_status(&binding.stage_id, config.allow_planned)?;
        }
        let out_dir = config.out_dir.clone();
        let explicit_stage_inputs = stage_artifact_input_policy(&pipeline_spec);
        let synthetic_stage_artifacts =
            synthetic_stage_artifact_policy(&pipeline_spec, &config.out_dir)?;
        let stage_dependencies = stage_dependency_policy(&pipeline_spec);
        let plans = crate::plan_compose::compose_fastq_stage_bindings_with_dependencies(
            &stage_bindings,
            &config.aux_images,
            config.adapter_bank.as_ref(),
            config.polyx_bank.as_ref(),
            config.contaminant_bank.as_ref(),
            config.enable_contaminant_removal,
            &config.r1,
            config.r2.as_deref(),
            config.reference_fasta.as_deref(),
            Some(&explicit_stage_inputs),
            Some(&synthetic_stage_artifacts),
            Some(&stage_dependencies),
            |binding, _r1, _r2| {
                let stage_dir = binding
                    .stage_instance_id
                    .as_deref()
                    .unwrap_or(binding.stage_id.as_str())
                    .trim_start_matches(STAGE_PREFIX);
                Ok(out_dir.join(stage_dir).join(binding.tool.tool_id.as_str()))
            },
        )?;
        let mut steps = plans
            .iter()
            .map(bijux_dna_stage_contract::execution_step_from_stage_plan)
            .collect::<Vec<_>>();
        let (compare_steps, compare_edges) = benchmark_compare_steps_for_toolsets(config, &plans)?;
        let (select_steps, select_edges, synthetic_step_nodes) =
            benchmark_select_steps_for_pipeline(config, &pipeline_spec, &plans)?;
        let mut edges =
            execution_edges_for_stage_plans(&pipeline_spec, &plans, &synthetic_step_nodes)?;
        steps.extend(compare_steps);
        edges.extend(compare_edges);
        steps.extend(select_steps);
        edges.extend(select_edges);
        let graph = ExecutionGraph::new(
            config.pipeline_id.clone(),
            PLANNER_VERSION,
            config.policy,
            steps,
            edges,
        )?;
        tracing::info!(
            target: "plan.graph",
            pipeline_id = %graph.pipeline_id(),
            steps = graph.steps().len(),
            edges = graph.edges().len(),
            "planned fastq execution graph"
        );
        Ok(graph)
    }

    /// # Errors
    /// Returns an error if benchmark fan-out planning fails.
    pub fn plan_stage_benchmark_cohort(
        config: &FastqStageBenchmarkConfig,
    ) -> Result<ExecutionGraph> {
        let stage_id = StageId::new(config.stage_id.clone());
        enforce_stage_status(stage_id.as_str(), config.allow_planned)?;
        if config.tools.is_empty() {
            return Err(anyhow!(
                "benchmark stage planning requires at least one tool for {}",
                stage_id.as_str()
            ));
        }

        let declared_bindings = crate::stage_api::toolset_for_stage(
            &stage_id,
            crate::stage_api::ToolsetExecutionMode::AllBindings,
        );
        let comparison_input_artifact_ids =
            bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&stage_id);
        let mut steps = Vec::new();
        let mut comparison_inputs = Vec::new();
        for tool in &config.tools {
            if !declared_bindings
                .iter()
                .any(|declared| declared == &tool.tool_id)
            {
                return Err(anyhow!(
                    "{} is not a declared binding for {}",
                    tool.tool_id.as_str(),
                    stage_id.as_str()
                ));
            }
            let maturity = crate::stage_api::stage_tool_maturity(&stage_id, &tool.tool_id)
                .ok_or_else(|| {
                    anyhow!(
                        "missing stage-tool maturity for {} / {}",
                        stage_id.as_str(),
                        tool.tool_id.as_str()
                    )
                })?;
            if maturity == crate::stage_api::StageToolMaturityLevel::PlannedBinding
                && !config.allow_planned
            {
                return Err(anyhow!(
                    "{} is a planned-only binding for {}; rerun with allow_planned to fan out planned tools",
                    tool.tool_id.as_str(),
                    stage_id.as_str()
                ));
            }
            let stage_bindings = [FastqStageBinding {
                stage_id: config.stage_id.clone(),
                stage_instance_id: None,
                tool: tool.clone(),
                reason: None,
                params: project_benchmark_stage_params_for_tool(&stage_id, &tool.tool_id, config.params.as_ref()),
            }];
            let stage_plans = compose_fastq_stage_bindings(
                &stage_bindings,
                &config.aux_images,
                config.adapter_bank.as_ref(),
                config.polyx_bank.as_ref(),
                config.contaminant_bank.as_ref(),
                config.enable_contaminant_removal,
                &config.r1,
                config.r2.as_deref(),
                config.reference_fasta.as_deref(),
                None,
                |binding, _r1, _r2| {
                    let stage_dir = binding.stage_id.trim_start_matches(STAGE_PREFIX);
                    Ok(config
                        .out_dir
                        .join(stage_dir)
                        .join(binding.tool.tool_id.as_str()))
                },
            )?;
            let Some(plan) = stage_plans.into_iter().next() else {
                return Err(anyhow!(
                    "benchmark stage planner produced no stage plan for {} / {}",
                    stage_id.as_str(),
                    tool.tool_id.as_str()
                ));
            };
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
                        tool.tool_id.as_str(),
                        output.name.as_str()
                    )),
                    output.path.clone(),
                    output.role,
                ));
            }
            steps.push(
                bijux_dna_stage_contract::execution_step_from_stage_plan_with_step_id(
                    &plan,
                    StepId::new(format!(
                        "{}.tool.{}",
                        stage_id.as_str(),
                        tool.tool_id.as_str()
                    )),
                ),
            );
        }

        let comparison_artifact_ids =
            bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&stage_id);
        if !comparison_artifact_ids.is_empty() {
            let compare_step_id = StepId::new(format!("{}.compare", stage_id.as_str()));
            let compare_out_dir = config
                .out_dir
                .join(stage_id.as_str().trim_start_matches(STAGE_PREFIX))
                .join("compare");
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
            steps.push(ExecutionStep {
                step_id: compare_step_id,
                stage_id: crate::STAGE_COMPARE_STAGE_TOOLS,
                command: CommandSpecV1 {
                    template: comparison_command,
                },
                image: ContainerImageRefV1 {
                    image: "bijux-dna-compare".to_string(),
                    digest: None,
                },
                resources: ToolConstraints::default(),
                io: StageIO {
                    inputs: comparison_inputs,
                    outputs: comparison_outputs,
                },
                out_dir: compare_out_dir,
                aux_images: BTreeMap::new(),
                expected_artifact_ids: comparison_artifact_ids
                    .iter()
                    .map(|artifact_id| ArtifactId::new(artifact_id.clone()))
                    .collect(),
                metrics_schema_ids: Vec::new(),
            });
        }

        let compare_step_id = StepId::new(format!("{}.compare", stage_id.as_str()));
        let edges = if steps.iter().any(|step| step.step_id == compare_step_id) {
            steps
                .iter()
                .filter(|step| step.step_id != compare_step_id)
                .map(|step| ExecutionEdge::new(step.step_id.clone(), compare_step_id.clone()))
                .collect()
        } else {
            Vec::new()
        };

        Ok(ExecutionGraph::new(
            config.pipeline_id.clone(),
            PLANNER_VERSION,
            config.policy,
            steps,
            edges,
        )?)
    }
}

fn project_benchmark_stage_params_for_tool(
    stage_id: &StageId,
    tool_id: &bijux_dna_core::ids::ToolId,
    params: Option<&FastqStageParameters>,
) -> Option<FastqStageParameters> {
    match (stage_id.as_str(), params) {
        (
            "fastq.correct_errors",
            Some(FastqStageParameters::CorrectErrors(params)),
        ) => Some(FastqStageParameters::CorrectErrors(
            project_correct_errors_params_for_tool(tool_id.as_str(), params),
        )),
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

fn comparison_command_for_stage(
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

fn comparison_artifact_file_name(artifact_id: &str) -> String {
    let stem = artifact_id.strip_suffix("_json").unwrap_or(artifact_id);
    format!("{stem}.json")
}

fn selection_artifact_file_name(artifact_id: &str) -> String {
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

fn inferred_selection_artifact_role(artifact_id: &str) -> ArtifactRole {
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

fn benchmark_compare_steps_for_toolsets(
    config: &FastqPlanConfig,
    plans: &[StagePlanV1],
) -> Result<(Vec<ExecutionStep>, Vec<ExecutionEdge>)> {
    if config.stage_toolsets.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut steps = Vec::new();
    let mut edges = Vec::new();
    for toolset in config
        .stage_toolsets
        .iter()
        .filter(|binding| binding.tools.len() > 1)
    {
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
        for plan in plans
            .iter()
            .filter(|plan| plan_originates_from_toolset(plan, toolset))
        {
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
                edges.push(ExecutionEdge::new(
                    step_id_for_plan(plan),
                    compare_step_id.clone(),
                ));
            }
            comparison_inputs.sort_by(|left, right| {
                left.name
                    .as_str()
                    .cmp(right.name.as_str())
                    .then_with(|| left.path.cmp(&right.path))
            });
            steps.push(ExecutionStep {
                step_id: compare_step_id,
                stage_id: crate::STAGE_COMPARE_STAGE_TOOLS,
                command: CommandSpecV1 {
                    template: comparison_command,
                },
                image: ContainerImageRefV1 {
                    image: "bijux-dna-compare".to_string(),
                    digest: None,
                },
                resources: ToolConstraints::default(),
                io: StageIO {
                    inputs: comparison_inputs,
                    outputs: comparison_outputs,
                },
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

type SelectStepPlan = (
    Vec<ExecutionStep>,
    Vec<ExecutionEdge>,
    std::collections::BTreeMap<String, StepId>,
);

fn benchmark_select_steps_for_pipeline(
    config: &FastqPlanConfig,
    pipeline_spec: &PipelineSpec,
    plans: &[StagePlanV1],
) -> Result<SelectStepPlan> {
    if !pipeline_spec.declares_graph_topology() {
        return Ok((Vec::new(), Vec::new(), std::collections::BTreeMap::new()));
    }

    let plan_by_node_id = plans
        .iter()
        .map(|plan| (step_id_for_plan(plan).as_str().to_string(), plan))
        .collect::<std::collections::BTreeMap<_, _>>();
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
        let incoming_edges = pipeline_spec
            .edges
            .iter()
            .filter(|edge| edge.to == node_id)
            .collect::<Vec<_>>();
        if incoming_edges.is_empty() {
            continue;
        }
        let source_plans = incoming_edges
            .iter()
            .map(|edge| {
                plan_by_node_id.get(&edge.from).copied().ok_or_else(|| {
                    anyhow!(
                        "selection node {} references unknown upstream step {}",
                        node_id,
                        edge.from
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let source_stage_id = StageId::new(source_plans[0].stage_id.as_str().to_string());
        if source_plans
            .iter()
            .any(|plan| plan.stage_id.as_str() != source_stage_id.as_str())
        {
            return Err(anyhow!(
                "selection node {} must join candidates from one stage family",
                node_id
            ));
        }
        let select_out_dir = config
            .out_dir
            .join(node_id.trim_start_matches(STAGE_PREFIX))
            .join("selected");
        let output_artifact_ids = pipeline_spec
            .edges
            .iter()
            .filter(|edge| edge.from == node_id)
            .filter_map(|edge| edge.from_output_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let outputs = output_artifact_ids
            .iter()
            .map(|artifact_id| {
                ArtifactRef::required(
                    ArtifactId::new(artifact_id.clone()),
                    select_out_dir.join(selection_artifact_file_name(artifact_id)),
                    inferred_selection_artifact_role(artifact_id),
                )
            })
            .collect::<Vec<_>>();
        let mut inputs = Vec::new();
        for edge in &incoming_edges {
            let source_plan = plan_by_node_id.get(&edge.from).copied().ok_or_else(|| {
                anyhow!(
                    "selection node {} references unresolved source plan {}",
                    node_id,
                    edge.from
                )
            })?;
            let source_output_id = edge.from_output_id.as_ref().ok_or_else(|| {
                anyhow!(
                    "selection node {} requires bound source output ids on incoming edges",
                    node_id
                )
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
            inputs.push(ArtifactRef::required(
                ArtifactId::new(edge.to_input_id.clone().ok_or_else(|| {
                    anyhow!(
                        "selection node {} requires bound destination input ids on incoming edges",
                        node_id
                    )
                })?),
                source_output.path.clone(),
                source_output.role,
            ));
        }
        inputs.sort_by(|left, right| {
            left.name
                .as_str()
                .cmp(right.name.as_str())
                .then_with(|| left.path.cmp(&right.path))
        });
        inputs.dedup_by(|left, right| left.name == right.name && left.path == right.path);
        let step_id = StepId::new(node_id.clone());
        steps.push(ExecutionStep {
            step_id: step_id.clone(),
            stage_id: crate::STAGE_SELECT_STAGE_TOOL,
            command: CommandSpecV1 {
                template: selection_command_for_stage(
                    &source_stage_id,
                    &output_artifact_ids,
                    config.selection_objective,
                ),
            },
            image: ContainerImageRefV1 {
                image: "bijux-dna-select".to_string(),
                digest: None,
            },
            resources: ToolConstraints::default(),
            io: StageIO { inputs, outputs },
            out_dir: select_out_dir,
            aux_images: BTreeMap::new(),
            expected_artifact_ids: output_artifact_ids
                .iter()
                .cloned()
                .map(ArtifactId::new)
                .collect(),
            metrics_schema_ids: Vec::new(),
        });
        step_nodes.insert(node_id, step_id);
    }

    Ok((steps, edges, step_nodes))
}

fn plan_originates_from_toolset(plan: &StagePlanV1, toolset: &FastqStageToolsetBinding) -> bool {
    if plan.stage_id.as_str() != toolset.stage_id {
        return false;
    }
    let plan_stage_instance_id = plan.stage_instance_id.as_ref().map(ToString::to_string);
    source_toolset_for_expanded_selection(
        std::slice::from_ref(toolset),
        plan.stage_id.as_str(),
        &plan_stage_instance_id,
    )
    .is_some()
}

fn compare_context_key_for_plan(plan: &StagePlanV1, stage_node_id: &str) -> String {
    let Some(assignments) = expanded_route_assignments(
        plan.stage_instance_id
            .as_ref()
            .map(|step_id| step_id.as_str()),
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
    let compare_dir = root_out_dir
        .join(stage_node_id.trim_start_matches(STAGE_PREFIX))
        .join("compare");
    if context_key.is_empty() {
        compare_dir
    } else {
        compare_dir.join(context_key)
    }
}

fn step_id_for_plan(plan: &StagePlanV1) -> StepId {
    plan.stage_instance_id
        .as_ref()
        .cloned()
        .unwrap_or_else(|| StepId::new(plan.stage_id.as_str().to_string()))
}

fn validate_select_stage_nodes(
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
        let incoming = pipeline_spec
            .edges
            .iter()
            .filter(|edge| edge.to == node_id)
            .collect::<Vec<_>>();
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
        for edge in pipeline_spec
            .edges
            .iter()
            .filter(|edge| edge.from == node_id)
        {
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

fn planner_owned_graph_stage(stage_id: &str) -> bool {
    stage_id == crate::STAGE_SELECT_STAGE_TOOL.as_str()
}

fn synthetic_stage_artifact_policy(
    pipeline_spec: &PipelineSpec,
    root_out_dir: &std::path::Path,
) -> Result<crate::plan_compose::SyntheticStageArtifactPolicy> {
    let mut artifacts = crate::plan_compose::SyntheticStageArtifactPolicy::new();
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
        let select_out_dir = root_out_dir
            .join(node_id.trim_start_matches(STAGE_PREFIX))
            .join("selected");
        artifacts.insert(
            node_id,
            artifact_ids
                .into_iter()
                .map(|artifact_id| {
                    ArtifactRef::required(
                        ArtifactId::new(artifact_id.clone()),
                        select_out_dir.join(selection_artifact_file_name(&artifact_id)),
                        inferred_selection_artifact_role(&artifact_id),
                    )
                })
                .collect(),
        );
    }
    Ok(artifacts)
}

fn normalize_stage_bindings(
    config: &FastqPlanConfig,
) -> Result<(PipelineSpec, Vec<FastqStageBinding>)> {
    if !config.stage_bindings.is_empty() {
        if !config.stage_toolsets.is_empty() {
            return Err(anyhow!(
                "FastqPlanConfig must use exactly one graph planning surface: stage_bindings or stage_toolsets"
            ));
        }
        ensure_unique_stage_binding_nodes(&config.stage_bindings)?;
        let pipeline_spec = config
            .pipeline_spec
            .clone()
            .map(Ok)
            .unwrap_or_else(|| implicit_pipeline_spec_from_bindings(&config.stage_bindings))?;
        return Ok((pipeline_spec, config.stage_bindings.clone()));
    }

    if !config.stage_toolsets.is_empty() {
        if !config.stage_bindings.is_empty() {
            return Err(anyhow!(
                "FastqPlanConfig must use exactly one graph planning surface: stage_bindings or stage_toolsets"
            ));
        }
        let base_pipeline = config
            .pipeline_spec
            .clone()
            .map(Ok)
            .unwrap_or_else(|| implicit_pipeline_spec_from_toolsets(&config.stage_toolsets))?;
        let toolsets = config
            .stage_toolsets
            .iter()
            .map(|binding| {
                if binding.tools.is_empty() {
                    return Err(anyhow!(
                        "stage toolset {} must include at least one tool",
                        binding.stage_id
                    ));
                }
                Ok(ToolsetSelection {
                    stage_id: binding.stage_id.clone(),
                    stage_instance_id: binding.stage_instance_id.clone(),
                    tool_ids: binding
                        .tools
                        .iter()
                        .map(|tool| tool.tool_id.to_string())
                        .collect(),
                    reason: binding.reason.clone().unwrap_or_default(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        if config
            .stage_toolsets
            .iter()
            .all(|binding| binding.tools.len() == 1)
        {
            let stage_bindings = config
                .stage_toolsets
                .iter()
                .map(|binding| FastqStageBinding {
                    stage_id: binding.stage_id.clone(),
                    stage_instance_id: binding.stage_instance_id.clone(),
                    tool: binding.tools[0].clone(),
                    reason: binding.reason.clone(),
                    params: binding.params.clone(),
                })
                .collect::<Vec<_>>();
            ensure_unique_stage_binding_nodes(&stage_bindings)?;
            return Ok((base_pipeline, stage_bindings));
        }
        let (expanded_pipeline, expanded_stage_tools) =
            expand_pipeline_stage_tool_routes(&base_pipeline, &toolsets)?;
        let stage_bindings = expanded_stage_tools
            .into_iter()
            .map(|selection| {
                let toolset = source_toolset_for_expanded_selection(
                    &config.stage_toolsets,
                    &selection.stage_id,
                    &selection.stage_instance_id,
                )
                .ok_or_else(|| {
                    anyhow!(
                        "expanded route binding {} missing source toolset definition",
                        PipelineSpec::stage_node_id(
                            &selection.stage_id,
                            selection.stage_instance_id.as_deref()
                        )
                    )
                })?;
                let tool = toolset
                    .tools
                    .iter()
                    .find(|tool| tool.tool_id.as_str() == selection.tool_id)
                    .cloned()
                    .ok_or_else(|| {
                        anyhow!(
                            "expanded route binding {} references undeclared tool {}",
                            selection.stage_id,
                            selection.tool_id
                        )
                    })?;
                Ok(FastqStageBinding {
                    stage_id: selection.stage_id,
                    stage_instance_id: selection.stage_instance_id,
                    tool,
                    reason: Some(selection.reason),
                    params: toolset.params.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        ensure_unique_stage_binding_nodes(&stage_bindings)?;
        return Ok((expanded_pipeline, stage_bindings));
    }

    Err(anyhow!(
        "FastqPlanConfig requires a graph-backed planning surface via stage_bindings or stage_toolsets"
    ))
}

fn implicit_pipeline_spec_from_bindings(bindings: &[FastqStageBinding]) -> Result<PipelineSpec> {
    implicit_pipeline_spec_from_nodes(
        bindings
            .iter()
            .map(|binding| PipelineNodeSpec {
                stage_id: binding.stage_id.clone(),
                stage_instance_id: binding.stage_instance_id.clone(),
            })
            .collect(),
        "stage_bindings",
    )
}

fn implicit_pipeline_spec_from_toolsets(
    toolsets: &[FastqStageToolsetBinding],
) -> Result<PipelineSpec> {
    implicit_pipeline_spec_from_nodes(
        toolsets
            .iter()
            .map(|binding| PipelineNodeSpec {
                stage_id: binding.stage_id.clone(),
                stage_instance_id: binding.stage_instance_id.clone(),
            })
            .collect(),
        "stage_toolsets",
    )
}

fn implicit_pipeline_spec_from_nodes(
    nodes: Vec<PipelineNodeSpec>,
    surface_name: &str,
) -> Result<PipelineSpec> {
    let mut node_id_by_stage_id = std::collections::BTreeMap::new();
    for node in &nodes {
        let stage_node_id =
            PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref());
        if node_id_by_stage_id
            .insert(node.stage_id.clone(), stage_node_id)
            .is_some()
        {
            return Err(anyhow!(
                "{surface_name} with repeated stage_id {} requires an explicit pipeline_spec",
                node.stage_id
            ));
        }
    }

    let stage_graph = preprocess_pipeline_graph_for_stage_order(
        &nodes
            .iter()
            .map(|node| StageId::new(node.stage_id.clone()))
            .collect::<Vec<_>>(),
    );
    let edges = stage_graph
        .edges
        .into_iter()
        .map(|edge| PipelineEdgeSpec {
            from: node_id_by_stage_id
                .get(&edge.from)
                .cloned()
                .unwrap_or(edge.from),
            to: node_id_by_stage_id
                .get(&edge.to)
                .cloned()
                .unwrap_or(edge.to),
            from_output_id: edge.from_output_id,
            to_input_id: edge.to_input_id,
        })
        .collect();
    Ok(PipelineSpec::graph(nodes, edges))
}

fn base_stage_instance_id(stage_instance_id: &Option<String>) -> Option<&str> {
    stage_instance_id
        .as_deref()
        .and_then(|value| value.split(".route.").next())
}

fn source_toolset_for_expanded_selection<'a>(
    toolsets: &'a [FastqStageToolsetBinding],
    stage_id: &str,
    expanded_stage_instance_id: &Option<String>,
) -> Option<&'a FastqStageToolsetBinding> {
    let base_instance_id = base_stage_instance_id(expanded_stage_instance_id);
    toolsets.iter().find(|binding| {
        binding.stage_id == stage_id
            && match binding.stage_instance_id.as_deref() {
                Some(stage_instance_id) => Some(stage_instance_id) == base_instance_id,
                None => {
                    base_instance_id.is_none()
                        || base_instance_id == Some(binding.stage_id.as_str())
                }
            }
    })
}

fn validate_reference_index_bindings(
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
                .map(|binding| binding.tool.tool_id.as_str());
                let dependency_backend = dependency_reference_index_binding(
                    binding,
                    &dependency_policy,
                    &binding_by_node_id,
                )?
                .map(|binding| binding.tool.tool_id.as_str());
                let Some(index_backend) = explicit_backend
                    .or(dependency_backend)
                    .or(current_index_backend)
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

fn binding_node_id(binding: &FastqStageBinding) -> String {
    binding
        .stage_instance_id
        .clone()
        .unwrap_or_else(|| binding.stage_id.clone())
}

fn stage_dependency_policy(
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
        dependencies
            .entry(edge.to.clone())
            .or_default()
            .push(edge.from.clone());
    }
    dependencies
}

fn explicit_reference_index_binding<'a>(
    binding: &FastqStageBinding,
    explicit_stage_inputs: &'a crate::plan_compose::StageArtifactInputPolicy,
    binding_by_node_id: &'a std::collections::BTreeMap<String, &'a FastqStageBinding>,
) -> Result<Option<&'a FastqStageBinding>> {
    Ok(explicit_stage_inputs
        .get(&binding_node_id(binding))
        .and_then(|inputs| {
            inputs
                .iter()
                .find(|input| input.to_input_id == "reference_index")
        })
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

fn execution_edges_for_stage_plans(
    pipeline_spec: &PipelineSpec,
    plans: &[StagePlanV1],
    synthetic_step_nodes: &std::collections::BTreeMap<String, StepId>,
) -> Result<Vec<ExecutionEdge>> {
    let mut plan_nodes = std::collections::BTreeMap::new();
    let mut stage_counts = std::collections::BTreeMap::new();
    for plan in plans {
        *stage_counts
            .entry(plan.stage_id.as_str().to_string())
            .or_insert(0usize) += 1;
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
        synthetic_step_nodes
            .iter()
            .map(|(node_id, step_id)| (node_id.clone(), step_id.clone())),
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
        .map(|edge| {
            (
                edge.from().as_str().to_string(),
                edge.to().as_str().to_string(),
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    edges.retain(|edge| {
        edge.from_output_id().is_some()
            || !artifact_bound_pairs.contains(&(
                edge.from().as_str().to_string(),
                edge.to().as_str().to_string(),
            ))
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

fn stage_artifact_input_policy(
    pipeline_spec: &PipelineSpec,
) -> crate::plan_compose::StageArtifactInputPolicy {
    let mut policies = crate::plan_compose::StageArtifactInputPolicy::new();
    if !pipeline_spec.declares_graph_topology() {
        return policies;
    }
    for edge in &pipeline_spec.edges {
        let (Some(from_output_id), Some(to_input_id)) = (&edge.from_output_id, &edge.to_input_id)
        else {
            continue;
        };
        policies.entry(edge.to.clone()).or_default().push(
            crate::plan_compose::StageArtifactInputBinding {
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
        anyhow!(
            "pipeline graph edge source {} does not resolve to a planned step",
            edge.from
        )
    })?;
    let to = plan_nodes.get(&edge.to).cloned().ok_or_else(|| {
        anyhow!(
            "pipeline graph edge target {} does not resolve to a planned step",
            edge.to
        )
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

fn ensure_unique_stage_binding_nodes(bindings: &[FastqStageBinding]) -> Result<()> {
    let mut seen_nodes = std::collections::BTreeSet::new();
    for binding in bindings {
        let node_id = binding
            .stage_instance_id
            .as_deref()
            .map(str::to_string)
            .unwrap_or_else(|| {
                format!(
                    "{}.tool.{}",
                    binding.stage_id,
                    binding.tool.tool_id.as_str()
                )
            });
        if !seen_nodes.insert(node_id.clone()) {
            return Err(anyhow!(
                "duplicate FASTQ stage node binding {}; repeated stage/tool bindings must set distinct stage_instance_id values",
                node_id
            ));
        }
    }
    Ok(())
}

fn stage_status(stage_id: &str) -> Option<String> {
    let stage_id = bijux_dna_core::ids::StageId::try_from(stage_id).ok()?;
    bijux_dna_domain_fastq::execution_support_for_stage(&stage_id).map(|support| {
        match support.execution_status {
            bijux_dna_domain_fastq::ExecutionStatus::Closed => "supported",
            bijux_dna_domain_fastq::ExecutionStatus::DeclaredOnly => "planned",
        }
        .to_string()
    })
}

fn enforce_stage_status(stage_id: &str, allow_planned: bool) -> Result<()> {
    match stage_status(stage_id).as_deref() {
        Some("supported") | None => Ok(()),
        Some("planned") | Some("out_of_scope") if allow_planned => Ok(()),
        Some("planned") | Some("out_of_scope") => Err(anyhow!(
            "stage {stage_id} is not active in current scope; re-run with --allow-planned to override"
        )),
        Some(other) => Err(anyhow!("stage {stage_id} has unknown status {other}")),
    }
}

#[derive(Debug, Clone)]
pub struct FastqPipelineInputs {
    pub policy: PlanPolicy,
    pub stage_toolsets: Vec<FastqStageToolsetBinding>,
    pub aux_images: BTreeMap<String, ContainerImageRefV1>,
    pub adapter_bank: Option<serde_json::Value>,
    pub polyx_bank: Option<serde_json::Value>,
    pub contaminant_bank: Option<serde_json::Value>,
    pub enable_contaminant_removal: bool,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub reference_fasta: Option<PathBuf>,
    pub out_dir: PathBuf,
}

fn stage_toolsets_for_pipeline_nodes(
    pipeline: &PipelineSpec,
    stage_toolsets: &[FastqStageToolsetBinding],
) -> Result<Vec<FastqStageToolsetBinding>> {
    let ordered_nodes = pipeline.ordered_nodes();
    if ordered_nodes.len() != stage_toolsets.len() {
        return Err(anyhow!(
            "pipeline nodes/toolset length mismatch: {} vs {}",
            ordered_nodes.len(),
            stage_toolsets.len()
        ));
    }

    ordered_nodes
        .into_iter()
        .zip(stage_toolsets.iter())
        .map(|(node, toolset)| {
            if node.stage_id != toolset.stage_id
                || node.stage_instance_id != toolset.stage_instance_id
            {
                return Err(anyhow!(
                    "graph toolsets must stay node-aligned; got pipeline node {} and toolset {}",
                    PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref()),
                    PipelineSpec::stage_node_id(
                        &toolset.stage_id,
                        toolset.stage_instance_id.as_deref()
                    ),
                ));
            }
            Ok(toolset.clone())
        })
        .collect::<Result<Vec<_>>>()
}

/// # Errors
/// Returns an error if planning fails.
#[allow(non_snake_case)]
pub fn plan_fastq_to_fastq__default__v1(
    inputs: &FastqPipelineInputs,
    options: DefaultPipelineOptions,
) -> Result<ExecutionGraph> {
    let pipeline = default_pipeline_spec(options);
    let stage_toolsets = stage_toolsets_for_pipeline_nodes(&pipeline, &inputs.stage_toolsets)?;
    let config = FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__default__v1".to_string(),
        policy: inputs.policy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: Some(pipeline.clone()),
        stage_bindings: Vec::new(),
        stage_toolsets,
        aux_images: inputs.aux_images.clone(),
        adapter_bank: inputs.adapter_bank.clone(),
        polyx_bank: inputs.polyx_bank.clone(),
        contaminant_bank: inputs.contaminant_bank.clone(),
        enable_contaminant_removal: inputs.enable_contaminant_removal,
        r1: inputs.r1.clone(),
        r2: inputs.r2.clone(),
        reference_fasta: inputs.reference_fasta.clone(),
        out_dir: inputs.out_dir.clone(),
        allow_planned: false,
    };
    FastqPlanner::plan(&config)
}

#[cfg(test)]
mod tests {
    use super::{
        implicit_pipeline_spec_from_bindings, implicit_pipeline_spec_from_toolsets,
        stage_toolsets_for_pipeline_nodes, FastqStageBinding, FastqStageToolsetBinding,
    };
    use anyhow::Result;
    use bijux_dna_core::contract::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec};
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
    };
    use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind};

    fn dummy_tool(tool_id: &str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::new(tool_id),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["echo".to_string(), tool_id.to_string()],
            },
            resources: ToolConstraints::default(),
        }
    }

    #[test]
    fn stage_toolsets_for_pipeline_nodes_preserves_node_identity() -> Result<()> {
        let pipeline = PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.validate_reads".to_string(),
                    stage_instance_id: Some("fastq.validate_reads.first".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                },
            ],
            vec![PipelineEdgeSpec {
                from: "fastq.validate_reads.first".to_string(),
                to: "fastq.trim_reads.fastp_branch".to_string(),
                from_output_id: None,
                to_input_id: None,
            }],
        );
        let toolsets = vec![
            FastqStageToolsetBinding {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: Some("fastq.validate_reads.first".to_string()),
                tools: vec![dummy_tool("fastqvalidator")],
                reason: Some(PlanDecisionReason::new(PlanReasonKind::Default, "validate")),
                params: None,
            },
            FastqStageToolsetBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                tools: vec![dummy_tool("fastp")],
                reason: Some(PlanDecisionReason::new(PlanReasonKind::Default, "trim")),
                params: None,
            },
        ];

        let toolsets = stage_toolsets_for_pipeline_nodes(&pipeline, &toolsets)?;

        assert_eq!(toolsets.len(), 2);
        assert_eq!(
            toolsets[0].stage_instance_id.as_deref(),
            Some("fastq.validate_reads.first")
        );
        assert_eq!(
            toolsets[1].stage_instance_id.as_deref(),
            Some("fastq.trim_reads.fastp_branch")
        );
        assert_eq!(toolsets[1].tools[0].tool_id.as_str(), "fastp");
        Ok(())
    }

    #[test]
    fn stage_toolsets_for_pipeline_nodes_rejects_toolset_mismatch() {
        let pipeline = PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.validate_reads".to_string(),
                    stage_instance_id: None,
                },
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: None,
                },
            ],
            Vec::new(),
        );

        let error = stage_toolsets_for_pipeline_nodes(
            &pipeline,
            &[FastqStageToolsetBinding {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: None,
                tools: vec![dummy_tool("fastqvalidator")],
                reason: None,
                params: None,
            }],
        )
        .expect_err("node/tool mismatch must fail");

        assert!(error
            .to_string()
            .contains("pipeline nodes/toolset length mismatch"));
    }

    #[test]
    fn implicit_pipeline_spec_from_bindings_uses_domain_dag_edges() -> Result<()> {
        let pipeline = implicit_pipeline_spec_from_bindings(&[
            FastqStageBinding {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: Some("fastq.validate_reads.entry".to_string()),
                tool: dummy_tool("fastqvalidator"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.profile_read_lengths".to_string(),
                stage_instance_id: Some("fastq.profile_read_lengths.metrics".to_string()),
                tool: dummy_tool("seqkit_stats"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.detect_adapters".to_string(),
                stage_instance_id: Some("fastq.detect_adapters.adapters".to_string()),
                tool: dummy_tool("fastqc"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.report_qc".to_string(),
                stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                tool: dummy_tool("multiqc"),
                reason: None,
                params: None,
            },
        ])?;

        assert_eq!(pipeline.nodes.len(), 4);
        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.validate_reads.entry"
                && edge.to == "fastq.profile_read_lengths.metrics"
        }));
        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.validate_reads.entry" && edge.to == "fastq.detect_adapters.adapters"
        }));
        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.profile_read_lengths.metrics"
                && edge.to == "fastq.report_qc.aggregate"
        }));
        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.detect_adapters.adapters" && edge.to == "fastq.report_qc.aggregate"
        }));
        Ok(())
    }

    #[test]
    fn implicit_pipeline_spec_from_toolsets_rejects_repeated_stage_ids() {
        let error = implicit_pipeline_spec_from_toolsets(&[
            FastqStageToolsetBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                tools: vec![dummy_tool("fastp")],
                reason: None,
                params: None,
            },
            FastqStageToolsetBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cutadapt_branch".to_string()),
                tools: vec![dummy_tool("cutadapt")],
                reason: None,
                params: None,
            },
        ])
        .expect_err("repeated stage ids must require an explicit graph");

        assert!(error
            .to_string()
            .contains("requires an explicit pipeline_spec"));
    }
}

/// # Errors
/// Returns an error if planning fails.
#[allow(non_snake_case)]
pub fn plan_fastq_to_bam__default__v1(
    stages: Vec<StagePlanV1>,
    policy: PlanPolicy,
) -> Result<ExecutionGraph> {
    let edges = default_edges_for_stages(&stages);
    let graph = ExecutionGraph::new(
        "fastq-to-bam__default__v1",
        PLANNER_VERSION,
        policy,
        stages
            .iter()
            .map(bijux_dna_stage_contract::execution_step_from_stage_plan)
            .collect(),
        edges
            .into_iter()
            .map(|edge| {
                ExecutionEdge::new(
                    StepId::new(edge.from().to_string()),
                    StepId::new(edge.to().to_string()),
                )
            })
            .collect(),
    )?;
    tracing::info!(
        target: "plan.graph",
        pipeline_id = %graph.pipeline_id(),
        steps = graph.steps().len(),
        edges = graph.edges().len(),
        "planned fastq-to-bam execution graph"
    );
    Ok(graph)
}

#[must_use]
pub fn cross_fastq_to_bam_id_catalog(profile_id: &str) -> Vec<String> {
    match profile_id {
        "fastq-to-bam__adna_shotgun__v1" | "fastq-to-bam__default__v1" => vec![
            STAGE_PREPROCESS_SUMMARY.as_str().to_string(),
            STAGE_CORE_PREPARE_REFERENCE.to_string(),
            BamStage::Align.as_str().to_string(),
            BamStage::QcPre.as_str().to_string(),
            BamStage::Coverage.as_str().to_string(),
            BamStage::Damage.as_str().to_string(),
        ],
        _ => Vec::new(),
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn compose_fastq_stage_bindings<F>(
    stage_bindings: &[FastqStageBinding],
    aux_images: &BTreeMap<String, ContainerImageRefV1>,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    enable_contaminant_removal: bool,
    r1: &std::path::Path,
    r2: Option<&std::path::Path>,
    reference_fasta: Option<&std::path::Path>,
    explicit_stage_inputs: Option<&crate::plan_compose::StageArtifactInputPolicy>,
    out_dir_for_stage: F,
) -> Result<Vec<bijux_dna_stage_contract::StagePlanV1>>
where
    F: FnMut(&FastqStageBinding, &std::path::Path, Option<&std::path::Path>) -> Result<PathBuf>,
{
    plan_compose::compose_fastq_stage_bindings(
        stage_bindings,
        aux_images,
        adapter_bank,
        polyx_bank,
        contaminant_bank,
        enable_contaminant_removal,
        r1,
        r2,
        reference_fasta,
        explicit_stage_inputs,
        out_dir_for_stage,
    )
}

#[derive(Debug, Clone)]
pub struct StageToolSelection {
    pub stage_id: String,
    pub stage_instance_id: Option<String>,
    pub tool_id: String,
    pub reason: PlanDecisionReason,
}

#[derive(Debug, Clone)]
pub struct ToolsetSelection {
    pub stage_id: String,
    pub stage_instance_id: Option<String>,
    pub tool_ids: Vec<String>,
    pub reason: PlanDecisionReason,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
struct RouteContext(std::collections::BTreeMap<String, String>);

#[derive(Debug, Clone)]
struct ExpandedRouteNode {
    expanded_node_id: String,
    input_context: RouteContext,
    output_context: RouteContext,
}

/// # Errors
/// Returns an error if toolset selection fails.
pub fn select_preprocess_toolsets(
    pipeline: &PipelineSpec,
    mode: crate::stage_api::ToolsetExecutionMode,
    allow_planned: bool,
) -> Result<Vec<ToolsetSelection>> {
    let mut selections = Vec::new();
    for node in pipeline.ordered_nodes() {
        if planner_owned_graph_stage(&node.stage_id) {
            continue;
        }
        enforce_stage_status(&node.stage_id, allow_planned)?;
        let stage_id = StageId::new(node.stage_id.clone());
        let tool_ids = crate::stage_api::toolset_for_stage(&stage_id, mode)
            .into_iter()
            .map(|tool_id| tool_id.to_string())
            .collect::<Vec<_>>();
        selections.push(ToolsetSelection {
            stage_id: node.stage_id,
            stage_instance_id: node.stage_instance_id,
            tool_ids,
            reason: PlanDecisionReason::new(
                PlanReasonKind::Default,
                match mode {
                    crate::stage_api::ToolsetExecutionMode::DefaultChoice => {
                        "selected default toolset"
                    }
                    crate::stage_api::ToolsetExecutionMode::GovernedExecution => {
                        "selected governed execution toolset"
                    }
                    crate::stage_api::ToolsetExecutionMode::BenchmarkCohort => {
                        "selected benchmark cohort toolset"
                    }
                    crate::stage_api::ToolsetExecutionMode::AllBindings => {
                        "selected declared binding toolset"
                    }
                },
            ),
        });
    }
    Ok(selections)
}

pub fn expand_pipeline_stage_tool_routes(
    pipeline: &PipelineSpec,
    toolsets: &[ToolsetSelection],
) -> Result<(PipelineSpec, Vec<StageToolSelection>)> {
    let ordered_nodes = pipeline.ordered_nodes();
    let executable_nodes = ordered_nodes
        .iter()
        .filter(|node| !planner_owned_graph_stage(&node.stage_id))
        .collect::<Vec<_>>();
    if executable_nodes.len() != toolsets.len() {
        return Err(anyhow!(
            "pipeline node/toolset length mismatch: {} vs {}",
            executable_nodes.len(),
            toolsets.len()
        ));
    }
    for (node, toolset) in executable_nodes.iter().zip(toolsets.iter()) {
        if node.stage_id != toolset.stage_id || node.stage_instance_id != toolset.stage_instance_id
        {
            return Err(anyhow!(
                "toolset expansion requires node-aligned stage selections; got pipeline node {} and toolset {}",
                PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref()),
                PipelineSpec::stage_node_id(&toolset.stage_id, toolset.stage_instance_id.as_deref()),
            ));
        }
        if toolset.tool_ids.is_empty() {
            return Err(anyhow!(
                "toolset expansion requires at least one tool for {}",
                node.stage_id
            ));
        }
    }
    let toolset_by_node_id = toolsets
        .iter()
        .map(|toolset| {
            (
                PipelineSpec::stage_node_id(
                    &toolset.stage_id,
                    toolset.stage_instance_id.as_deref(),
                ),
                toolset,
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    let route_count = toolsets.iter().try_fold(1usize, |count, toolset| {
        count
            .checked_mul(toolset.tool_ids.len())
            .ok_or_else(|| anyhow!("preprocess tool route expansion overflowed route count"))
    })?;
    let max_route_specific_pipelines = max_route_specific_pipelines()?;
    if route_count > max_route_specific_pipelines {
        return Err(anyhow!(
            "preprocess tool route expansion would create {route_count} route-specific pipelines; configured limit is {max_route_specific_pipelines}. Narrow the stage toolsets or raise BIJUX_FASTQ_MAX_ROUTE_PIPELINES"
        ));
    }

    let base_edges = if pipeline.declares_graph_topology() {
        pipeline.edges.clone()
    } else {
        ordered_nodes
            .windows(2)
            .map(|window| PipelineEdgeSpec {
                from: PipelineSpec::stage_node_id(
                    &window[0].stage_id,
                    window[0].stage_instance_id.as_deref(),
                ),
                to: PipelineSpec::stage_node_id(
                    &window[1].stage_id,
                    window[1].stage_instance_id.as_deref(),
                ),
                from_output_id: None,
                to_input_id: None,
            })
            .collect::<Vec<_>>()
    };

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut selections = Vec::new();
    let mut expanded_nodes_by_original =
        std::collections::BTreeMap::<String, Vec<ExpandedRouteNode>>::new();
    let predecessor_sets = predecessor_context_sets(&base_edges);
    for node in ordered_nodes {
        let node_id =
            PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref());
        let input_contexts =
            incoming_route_contexts(&node_id, &predecessor_sets, &expanded_nodes_by_original)?;
        if planner_owned_graph_stage(&node.stage_id) {
            let collapsed_source_nodes = collapsed_source_nodes_for_select(&node_id, &base_edges)?;
            for input_context in input_contexts {
                let output_context = input_context.without(&collapsed_source_nodes);
                let expanded_node_id = expanded_planner_stage_instance_id(
                    &node.stage_id,
                    node.stage_instance_id.as_deref(),
                    &output_context.route_key(),
                );
                nodes.push(PipelineNodeSpec {
                    stage_id: node.stage_id.clone(),
                    stage_instance_id: Some(expanded_node_id.clone()),
                });
                expanded_nodes_by_original
                    .entry(node_id.clone())
                    .or_default()
                    .push(ExpandedRouteNode {
                        expanded_node_id,
                        input_context,
                        output_context,
                    });
            }
            continue;
        }

        let toolset = toolset_by_node_id
            .get(&node_id)
            .copied()
            .ok_or_else(|| anyhow!("toolset expansion requires a stage toolset for {}", node_id))?;
        for input_context in input_contexts {
            for tool_id in &toolset.tool_ids {
                let output_context =
                    input_context.with_assignment(node_id.clone(), tool_id.clone());
                let stage_instance_id = expanded_stage_instance_id(
                    &node.stage_id,
                    node.stage_instance_id.as_deref(),
                    tool_id,
                    &output_context.route_key(),
                );
                nodes.push(PipelineNodeSpec {
                    stage_id: node.stage_id.clone(),
                    stage_instance_id: Some(stage_instance_id.clone()),
                });
                selections.push(StageToolSelection {
                    stage_id: node.stage_id.clone(),
                    stage_instance_id: Some(stage_instance_id.clone()),
                    tool_id: tool_id.clone(),
                    reason: toolset.reason.clone(),
                });
                expanded_nodes_by_original
                    .entry(node_id.clone())
                    .or_default()
                    .push(ExpandedRouteNode {
                        expanded_node_id: stage_instance_id,
                        input_context: input_context.clone(),
                        output_context,
                    });
            }
        }
    }

    for edge in &base_edges {
        let from_nodes = expanded_nodes_by_original
            .get(&edge.from)
            .ok_or_else(|| anyhow!("expanded route missing source node {}", edge.from))?;
        let to_nodes = expanded_nodes_by_original
            .get(&edge.to)
            .ok_or_else(|| anyhow!("expanded route missing target node {}", edge.to))?;
        for from_node in from_nodes {
            for to_node in to_nodes {
                if !from_node
                    .output_context
                    .is_subset_of(&to_node.input_context)
                {
                    continue;
                }
                edges.push(PipelineEdgeSpec {
                    from: from_node.expanded_node_id.clone(),
                    to: to_node.expanded_node_id.clone(),
                    from_output_id: edge.from_output_id.clone(),
                    to_input_id: expanded_to_input_id(edge, from_node),
                });
            }
        }
    }

    nodes.sort_by(|left, right| {
        PipelineSpec::stage_node_id(&left.stage_id, left.stage_instance_id.as_deref()).cmp(
            &PipelineSpec::stage_node_id(&right.stage_id, right.stage_instance_id.as_deref()),
        )
    });
    nodes.dedup_by(|left, right| {
        left.stage_id == right.stage_id && left.stage_instance_id == right.stage_instance_id
    });
    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| left.from_output_id.cmp(&right.from_output_id))
            .then_with(|| left.to_input_id.cmp(&right.to_input_id))
    });
    edges.dedup_by(|left, right| {
        left.from == right.from
            && left.to == right.to
            && left.from_output_id == right.from_output_id
            && left.to_input_id == right.to_input_id
    });
    selections.sort_by(|left, right| {
        left.stage_instance_id
            .cmp(&right.stage_instance_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    selections.dedup_by(|left, right| {
        left.stage_instance_id == right.stage_instance_id && left.tool_id == right.tool_id
    });

    Ok((PipelineSpec::graph(nodes, edges), selections))
}

fn max_route_specific_pipelines() -> Result<usize> {
    let Some(raw) = std::env::var_os("BIJUX_FASTQ_MAX_ROUTE_PIPELINES") else {
        return Ok(DEFAULT_MAX_ROUTE_SPECIFIC_PIPELINES);
    };
    let parsed = raw.to_string_lossy().parse::<usize>().map_err(|error| {
        anyhow!("BIJUX_FASTQ_MAX_ROUTE_PIPELINES must be a positive integer: {error}")
    })?;
    if parsed == 0 {
        return Err(anyhow!(
            "BIJUX_FASTQ_MAX_ROUTE_PIPELINES must be greater than zero"
        ));
    }
    Ok(parsed)
}

fn predecessor_context_sets(
    edges: &[PipelineEdgeSpec],
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut predecessors = std::collections::BTreeMap::<String, Vec<String>>::new();
    for edge in edges {
        predecessors
            .entry(edge.to.clone())
            .or_default()
            .push(edge.from.clone());
    }
    for upstream_nodes in predecessors.values_mut() {
        upstream_nodes.sort();
        upstream_nodes.dedup();
    }
    predecessors
}

fn incoming_route_contexts(
    node_id: &str,
    predecessor_sets: &std::collections::BTreeMap<String, Vec<String>>,
    expanded_nodes_by_original: &std::collections::BTreeMap<String, Vec<ExpandedRouteNode>>,
) -> Result<Vec<RouteContext>> {
    let Some(predecessors) = predecessor_sets.get(node_id) else {
        return Ok(vec![RouteContext::default()]);
    };
    let predecessor_context_sets = predecessors
        .iter()
        .map(|predecessor| {
            expanded_nodes_by_original
                .get(predecessor)
                .ok_or_else(|| anyhow!("expanded route missing predecessor node {}", predecessor))
                .map(|nodes| {
                    nodes
                        .iter()
                        .map(|node| node.output_context.clone())
                        .collect::<Vec<_>>()
                })
        })
        .collect::<Result<Vec<_>>>()?;
    combine_route_context_sets(&predecessor_context_sets)
}

fn combine_route_context_sets(context_sets: &[Vec<RouteContext>]) -> Result<Vec<RouteContext>> {
    let mut combined = vec![RouteContext::default()];
    for contexts in context_sets {
        let mut next = Vec::new();
        for prior in &combined {
            for context in contexts {
                if let Some(merged) = prior.merge(context) {
                    next.push(merged);
                }
            }
        }
        if next.is_empty() {
            return Err(anyhow!(
                "toolset route expansion found incompatible branch contexts while rejoining graph inputs"
            ));
        }
        next.sort();
        next.dedup();
        combined = next;
    }
    Ok(combined)
}

fn collapsed_source_nodes_for_select(
    select_node_id: &str,
    edges: &[PipelineEdgeSpec],
) -> Result<std::collections::BTreeSet<String>> {
    let incoming_sources = edges
        .iter()
        .filter(|edge| edge.to == select_node_id)
        .map(|edge| edge.from.clone())
        .collect::<std::collections::BTreeSet<_>>();
    if incoming_sources.is_empty() {
        return Err(anyhow!(
            "selection node {} requires incoming candidate edges",
            select_node_id
        ));
    }
    if incoming_sources.len() > 1 {
        return Err(anyhow!(
            "toolset route expansion requires selection node {} to collapse one source stage node; use explicit stage_bindings for multi-source selection joins",
            select_node_id
        ));
    }
    Ok(incoming_sources)
}

fn expanded_planner_stage_instance_id(
    stage_id: &str,
    stage_instance_id: Option<&str>,
    route_key: &str,
) -> String {
    let base_node_id = stage_instance_id.unwrap_or(stage_id);
    if route_key.is_empty() {
        base_node_id.to_string()
    } else {
        format!("{base_node_id}.route.{route_key}")
    }
}

fn expanded_to_input_id(edge: &PipelineEdgeSpec, from_node: &ExpandedRouteNode) -> Option<String> {
    let base_input_id = edge.to_input_id.clone()?;
    if !edge.to.starts_with(crate::STAGE_SELECT_STAGE_TOOL.as_str()) {
        return Some(base_input_id);
    }
    let tool_id = from_node
        .output_context
        .0
        .get(&edge.from)
        .cloned()
        .unwrap_or_else(|| "candidate".to_string());
    Some(format!("{tool_id}_{base_input_id}"))
}

fn expanded_stage_instance_id(
    stage_id: &str,
    stage_instance_id: Option<&str>,
    tool_id: &str,
    route_key: &str,
) -> String {
    let base_node_id = stage_instance_id.unwrap_or(stage_id);
    format!("{base_node_id}.route.{route_key}.tool.{tool_id}")
}


impl RouteContext {
    fn with_assignment(&self, node_id: String, tool_id: String) -> Self {
        let mut assignments = self.0.clone();
        assignments.insert(node_id, tool_id);
        Self(assignments)
    }

    fn without(&self, node_ids: &std::collections::BTreeSet<String>) -> Self {
        let mut assignments = self.0.clone();
        for node_id in node_ids {
            assignments.remove(node_id);
        }
        Self(assignments)
    }

    fn merge(&self, other: &Self) -> Option<Self> {
        let mut assignments = self.0.clone();
        for (node_id, tool_id) in &other.0 {
            match assignments.get(node_id) {
                Some(existing) if existing != tool_id => return None,
                Some(_) => {}
                None => {
                    assignments.insert(node_id.clone(), tool_id.clone());
                }
            }
        }
        Some(Self(assignments))
    }

    fn route_key(&self) -> String {
        self.0
            .iter()
            .map(|(node_id, tool_id)| format!("{node_id}={tool_id}"))
            .collect::<Vec<_>>()
            .join("__")
    }

    fn is_subset_of(&self, other: &Self) -> bool {
        self.0
            .iter()
            .all(|(node_id, tool_id)| other.0.get(node_id) == Some(tool_id))
    }
}

/// # Errors
/// Returns an error if node-aware tool selection fails.
pub fn select_preprocess_stage_tools(
    registry: &bijux_dna_core::contract::ToolRegistry,
    pipeline: &PipelineSpec,
    args: &crate::selection::args::BenchFastqPreprocessArgs,
    bench_repo: Option<&dyn BenchResultsRepository>,
) -> Result<Vec<StageToolSelection>> {
    let executable_nodes = pipeline
        .ordered_nodes()
        .into_iter()
        .filter(|node| !planner_owned_graph_stage(&node.stage_id))
        .collect::<Vec<_>>();
    let paired_end = args.r2.is_some();
    let mut selected_tools: Vec<StageToolSelection> = executable_nodes
        .iter()
        .map(|node| {
            let stage_id = StageId::new(node.stage_id.clone());
            let compatible_tools = crate::stage_api::filter_tools_for_input_layout(
                &stage_id,
                registry
                    .tools_for_stage(&stage_id)
                    .iter()
                    .map(|tool| tool.tool_id.clone())
                    .collect(),
                paired_end,
            );
            let tool_id = crate::selection::default_tool_for_stage(&stage_id)
                .filter(|tool_id| {
                    crate::stage_api::tool_supports_input_layout(&stage_id, tool_id, paired_end)
                })
                .or_else(|| compatible_tools.first().cloned())
                .map(|tool| tool.to_string())
                .ok_or_else(|| anyhow!("no layout-compatible tool for stage {}", node.stage_id))?;
            Ok(StageToolSelection {
                stage_id: node.stage_id.clone(),
                stage_instance_id: node.stage_instance_id.clone(),
                tool_id,
                reason: PlanDecisionReason::new(
                    PlanReasonKind::Default,
                    "default tool from pipeline catalog",
                ),
            })
        })
        .collect::<Result<_>>()?;

    if args.auto {
        let corpus_id = args
            .bench_corpus
            .ok_or_else(|| anyhow!("--bench-corpus is required with --auto"))?;
        let corpus = bijux_dna_domain_fastq::bench_corpus(corpus_id);
        let objective = bijux_dna_core::contract::objective_spec(args.objective);
        let repo = bench_repo.ok_or_else(|| {
            anyhow!("bench results repository required for --auto tool selection")
        })?;
        let mut selections = Vec::new();
        for (idx, node) in executable_nodes.iter().enumerate() {
            let stage_id = bijux_dna_core::ids::StageId::new(node.stage_id.clone());
            let prior_stage_ids = selected_tools[..idx]
                .iter()
                .map(|selection| selection.stage_id.clone())
                .collect::<Vec<_>>();
            let query_context = bench_query_context_for_preprocess_stage(
                &stage_id,
                args,
                &prior_stage_ids,
                &selected_tools[..idx],
            )?;
            let tool_ids = registry
                .tools_for_stage(&stage_id)
                .iter()
                .map(|tool| tool.tool_id.clone())
                .collect::<Vec<_>>();
            let tool_ids =
                crate::stage_api::filter_tools_for_input_layout(&stage_id, tool_ids, paired_end)
                    .into_iter()
                    .map(|tool| tool.to_string())
                    .collect::<Vec<_>>();
            let mut tool_records = Vec::new();
            for tool in &tool_ids {
                let records = repo.bench_results(&stage_id, tool, &corpus, &query_context)?;
                tool_records.push((tool.clone(), records));
            }
            let selection = bijux_dna_core::contract::select_stage(
                &stage_id,
                &tool_records,
                &objective,
                args.allow_partial,
            );
            if let Some(selected) = selection.selected.as_ref() {
                selected_tools[idx] = StageToolSelection {
                    stage_id: node.stage_id.clone(),
                    stage_instance_id: node.stage_instance_id.clone(),
                    tool_id: selected.clone(),
                    reason: PlanDecisionReason::new(
                        PlanReasonKind::InputAssessed,
                        "auto-selected from benchmark corpus",
                    ),
                };
            }
            selections.push(selection);
        }
    }

    Ok(selected_tools)
}

fn bench_query_context_for_stage(
    stage_id: &bijux_dna_core::ids::StageId,
) -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    bijux_dna_domain_fastq::governed_stage_bench_query_context(stage_id.as_str()).map_err(|err| {
        anyhow!(
            "compute benchmark query context for {}: {err}",
            stage_id.as_str()
        )
    })
}

fn bench_query_context_for_preprocess_stage(
    stage_id: &bijux_dna_core::ids::StageId,
    args: &crate::selection::args::BenchFastqPreprocessArgs,
    prior_stages: &[String],
    prior_tools: &[StageToolSelection],
) -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    let mut context = bench_query_context_for_stage(stage_id)?;
    if let Some(reference_fasta) = args.reference_fasta.as_ref() {
        context = context.with_reference_hash(
            bijux_dna_infra::hash_file_sha256(reference_fasta).map_err(|err| {
                anyhow!(
                    "hash reference FASTA for benchmark query context {}: {err}",
                    reference_fasta.display()
                )
            })?,
        );
    }
    for (bank_id, bank_hash) in bank_hashes_for_preprocess_args(args)? {
        context = context.with_bank_hash(bank_id, bank_hash);
    }
    let lineage_hash = prior_stages
        .iter()
        .zip(prior_tools.iter())
        .map(|(stage_id, tool)| format!("{stage_id}={}", tool.tool_id))
        .collect::<Vec<_>>()
        .join("|");
    if !lineage_hash.is_empty() {
        context = context.with_lineage_hash(lineage_hash);
    }
    Ok(context)
}

fn bank_hashes_for_preprocess_args(
    args: &crate::selection::args::BenchFastqPreprocessArgs,
) -> Result<Vec<(String, String)>> {
    let mut hashes = Vec::new();
    if args.adapter_bank_preset.is_some()
        || args.adapter_bank.is_some()
        || args.adapter_bank_file.is_some()
        || !args.enable_adapters.is_empty()
        || !args.disable_adapters.is_empty()
    {
        if let Some(context) = bijux_dna_domain_fastq::banks::adapter_bank_context(
            args.adapter_bank_preset.as_deref(),
            args.adapter_bank.as_deref(),
            args.adapter_bank_file.as_deref(),
            &args.enable_adapters,
            &args.disable_adapters,
        )? {
            if let Some(bank_hash) = context.get("bank_hash").and_then(serde_json::Value::as_str) {
                hashes.push(("adapter_bank".to_string(), bank_hash.to_string()));
            }
        }
    }
    if args.polyx_preset.is_some() {
        if let Some(context) =
            bijux_dna_domain_fastq::banks::polyx_bank_context(args.polyx_preset.as_deref())?
        {
            if let Some(bank_hash) = context.get("bank_hash").and_then(serde_json::Value::as_str) {
                hashes.push(("polyx_bank".to_string(), bank_hash.to_string()));
            }
        }
    }
    if args.contaminant_preset.is_some() {
        if let Some(context) = bijux_dna_domain_fastq::banks::contaminant_bank_context(
            args.contaminant_preset.as_deref(),
        )? {
            if let Some(bank_hash) = context.get("bank_hash").and_then(serde_json::Value::as_str) {
                hashes.push(("contaminant_bank".to_string(), bank_hash.to_string()));
            }
        }
    }
    hashes.sort();
    Ok(hashes)
}

include!("../tool_selection_facade.rs");

#[must_use]
pub fn scale_tool_spec_for_jobs(tool: &ToolExecutionSpecV1, jobs: usize) -> ToolExecutionSpecV1 {
    if jobs <= 1 {
        return tool.clone();
    }
    let mut scaled = tool.clone();
    let threads = scaled.resources.threads;
    let denom = u32::try_from(jobs).unwrap_or(1);
    scaled.resources.threads = (threads / denom).max(1);
    scaled
}

#[cfg(test)]
#[path = "../unit_checks.rs"]
mod unit_checks;
