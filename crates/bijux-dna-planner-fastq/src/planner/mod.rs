use std::collections::BTreeMap;
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
use bijux_dna_domain_fastq::STAGE_PREFIX;
use bijux_dna_pipelines::STAGE_CORE_PREPARE_REFERENCE;
use bijux_dna_stage_contract::{
    default_edges_for_stages, PlanDecisionReason, PlanReasonKind, StagePlanV1,
};

use crate::{
    compose, default_pipeline_spec, BenchResultsRepository, DefaultPipelineOptions,
    PLANNER_VERSION, STAGE_PREPROCESS_SUMMARY,
};

mod benchmark;
mod graph_policy;
mod layout_branching;
mod local_readiness;
mod local_smoke;
mod quality_sampling;
mod route_expansion;
mod selection_planning;
mod types;

pub use crate::selection::{
    apply_tool_overrides, apply_toolset_overrides, fastq_pipeline_id_catalog,
    select_cluster_otus_tools, select_correct_tools, select_deplete_host_tools,
    select_deplete_reference_contaminants_tools, select_deplete_rrna_tools,
    select_detect_adapters_tools, select_filter_low_complexity_tools, select_filter_tools,
    select_index_reference_tools, select_infer_asvs_tools, select_merge_tools,
    select_normalize_abundance_tools, select_normalize_primers_tools,
    select_profile_overrepresented_tools, select_profile_read_lengths_tools, select_qc_post_tools,
    select_remove_chimeras_tools, select_remove_duplicates_tools, select_screen_tools,
    select_stats_tools, select_trim_tools, select_umi_tools, select_validate_tools,
};
use benchmark::{
    benchmark_compare_steps_for_toolsets, benchmark_select_steps_for_pipeline,
    comparison_artifact_file_name, comparison_command_for_stage, inferred_selection_artifact_role,
    project_benchmark_stage_params_for_tool, selection_artifact_file_name,
};
use graph_policy::{
    enforce_stage_status, ensure_unique_stage_binding_nodes, execution_edges_for_stage_plans,
    planner_owned_graph_stage, stage_artifact_input_policy, stage_dependency_policy,
    synthetic_stage_artifact_policy, validate_reference_index_bindings,
    validate_select_stage_nodes,
};
pub(crate) use layout_branching::apply_layout_branching;
pub use local_readiness::{local_deplete_rrna_plan, local_index_reference_plan};
pub use local_smoke::{
    local_detect_adapters_smoke_plans, local_detect_duplicates_premerge_smoke_plans,
    local_estimate_library_complexity_prealign_smoke_plans,
    local_filter_low_complexity_smoke_plans, local_filter_reads_smoke_plans,
    local_merge_pairs_smoke_plans, local_normalize_primers_smoke_plans,
    local_profile_read_lengths_smoke_plans,
    local_profile_reads_smoke_plans, local_remove_duplicates_smoke_plans,
    local_trim_polyg_tails_smoke_plans,
    local_trim_reads_smoke_plans, local_trim_terminal_damage_smoke_plans,
    local_validate_reads_smoke_plans, LocalDetectAdaptersSmokeCasePlan,
    LocalDetectDuplicatesPremergeSmokeCasePlan,
    LocalEstimateLibraryComplexityPrealignSmokeCasePlan,
    LocalFilterLowComplexitySmokeCasePlan, LocalFilterReadsSmokeCasePlan,
    LocalMergePairsSmokeCasePlan, LocalNormalizePrimersSmokeCasePlan,
    LocalProfileReadLengthsSmokeCasePlan, LocalRemoveDuplicatesSmokeCasePlan,
    LocalProfileReadsSmokeCasePlan, LocalTrimPolygTailsSmokeCasePlan, LocalTrimReadsSmokeCasePlan,
    LocalTrimTerminalDamageSmokeCasePlan, LocalValidateReadsSmokeCasePlan,
};
pub(crate) use quality_sampling::estimate_mean_q;
pub use route_expansion::{expand_pipeline_stage_tool_routes, select_preprocess_toolsets};
pub use route_expansion::{StageToolSelection, ToolsetSelection};
pub use selection_planning::select_preprocess_stage_tools;
pub use types::*;

pub struct FastqPlanner;

const DEFAULT_MAX_ROUTE_SPECIFIC_PIPELINES: usize = 4096;

impl FastqPlanner {
    /// # Errors
    /// Returns an error if planning fails or the plan lint fails.
    pub fn plan(config: &FastqPlanConfig) -> Result<ExecutionGraph> {
        let plans = plan_fastq_stage_plans(config)?;
        let (pipeline_spec, _) = normalize_stage_bindings(config)?;
        let (compare_steps, compare_edges) = benchmark_compare_steps_for_toolsets(config, &plans)?;
        let (select_steps, select_edges, synthetic_step_nodes) =
            benchmark_select_steps_for_pipeline(config, &pipeline_spec, &plans)?;
        let mut edges =
            execution_edges_for_stage_plans(&pipeline_spec, &plans, &synthetic_step_nodes)?;
        let mut steps = plans
            .iter()
            .map(bijux_dna_stage_contract::execution_step_from_stage_plan)
            .collect::<Vec<_>>();
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
    /// Returns an error if stage planning fails before graph materialization.
    pub fn plan_stage_plans(config: &FastqPlanConfig) -> Result<Vec<StagePlanV1>> {
        plan_fastq_stage_plans(config)
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

        let declared_bindings = stage_benchmark_declared_bindings(&stage_id);
        let comparison_input_artifact_ids =
            bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&stage_id);
        let mut steps = Vec::new();
        let mut comparison_inputs = Vec::new();
        for tool in &config.tools {
            ensure_stage_benchmark_tool(config, &stage_id, &declared_bindings, tool)?;
            let plan = stage_benchmark_plan_for_tool(config, &stage_id, tool)?;
            comparison_inputs.extend(stage_benchmark_comparison_inputs(
                tool,
                &plan,
                &comparison_input_artifact_ids,
            ));
            steps.push(bijux_dna_stage_contract::execution_step_from_stage_plan_with_step_id(
                &plan,
                StepId::new(format!("{}.tool.{}", stage_id.as_str(), tool.tool_id.as_str())),
            ));
        }

        let compare_step_id = StepId::new(format!("{}.compare", stage_id.as_str()));
        if let Some(compare_step) = stage_benchmark_comparison_step(
            config,
            &stage_id,
            compare_step_id.clone(),
            comparison_inputs,
        )? {
            steps.push(compare_step);
        }
        let edges = stage_benchmark_comparison_edges(&steps, &compare_step_id);

        Ok(ExecutionGraph::new(
            config.pipeline_id.clone(),
            PLANNER_VERSION,
            config.policy,
            steps,
            edges,
        )?)
    }
}

/// # Errors
/// Returns an error if stage planning fails before graph materialization.
pub fn plan_fastq_stage_plans(config: &FastqPlanConfig) -> Result<Vec<StagePlanV1>> {
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
    let plans = crate::compose::compose_fastq_stage_bindings_with_dependencies(
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
    Ok(plans)
}

fn stage_benchmark_declared_bindings(stage_id: &StageId) -> Vec<bijux_dna_core::ids::ToolId> {
    crate::stage_api::toolset_for_stage(
        stage_id,
        crate::stage_api::ToolsetExecutionMode::AllBindings,
    )
}

fn ensure_stage_benchmark_tool(
    config: &FastqStageBenchmarkConfig,
    stage_id: &StageId,
    declared_bindings: &[bijux_dna_core::ids::ToolId],
    tool: &ToolExecutionSpecV1,
) -> Result<()> {
    if !declared_bindings.iter().any(|declared| declared == &tool.tool_id) {
        return Err(anyhow!(
            "{} is not a declared binding for {}",
            tool.tool_id.as_str(),
            stage_id.as_str()
        ));
    }
    let maturity =
        crate::stage_api::stage_tool_maturity(stage_id, &tool.tool_id).ok_or_else(|| {
            anyhow!(
                "missing stage-tool maturity for {} / {}",
                stage_id.as_str(),
                tool.tool_id.as_str()
            )
        })?;
    if maturity == crate::stage_api::StageToolMaturityLevel::PlannedBinding && !config.allow_planned
    {
        return Err(anyhow!(
            "{} is a planned-only binding for {}; rerun with allow_planned to fan out planned tools",
            tool.tool_id.as_str(),
            stage_id.as_str()
        ));
    }
    Ok(())
}

fn stage_benchmark_plan_for_tool(
    config: &FastqStageBenchmarkConfig,
    stage_id: &StageId,
    tool: &ToolExecutionSpecV1,
) -> Result<StagePlanV1> {
    let stage_bindings = [FastqStageBinding {
        stage_id: config.stage_id.clone(),
        stage_instance_id: None,
        tool: tool.clone(),
        reason: None,
        params: project_benchmark_stage_params_for_tool(
            stage_id,
            &tool.tool_id,
            config.params.as_ref(),
        ),
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
            Ok(config.out_dir.join(stage_dir).join(binding.tool.tool_id.as_str()))
        },
    )?;
    stage_plans.into_iter().next().ok_or_else(|| {
        anyhow!(
            "benchmark stage planner produced no stage plan for {} / {}",
            stage_id.as_str(),
            tool.tool_id.as_str()
        )
    })
}

fn stage_benchmark_comparison_inputs(
    tool: &ToolExecutionSpecV1,
    plan: &StagePlanV1,
    comparison_input_artifact_ids: &[String],
) -> Vec<ArtifactRef> {
    plan.io
        .outputs
        .iter()
        .filter(|output| {
            comparison_input_artifact_ids.is_empty()
                || comparison_input_artifact_ids
                    .iter()
                    .any(|artifact_id| *artifact_id == output.name.as_str())
        })
        .map(|output| {
            ArtifactRef::required(
                ArtifactId::new(format!("{}__{}", tool.tool_id.as_str(), output.name.as_str())),
                output.path.clone(),
                output.role,
            )
        })
        .collect()
}

fn stage_benchmark_comparison_step(
    config: &FastqStageBenchmarkConfig,
    stage_id: &StageId,
    compare_step_id: StepId,
    comparison_inputs: Vec<ArtifactRef>,
) -> Result<Option<ExecutionStep>> {
    let comparison_artifact_ids =
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(stage_id);
    if comparison_artifact_ids.is_empty() {
        return Ok(None);
    }
    let compare_out_dir =
        config.out_dir.join(stage_id.as_str().trim_start_matches(STAGE_PREFIX)).join("compare");
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
    Ok(Some(ExecutionStep {
        step_id: compare_step_id,
        stage_id: crate::STAGE_COMPARE_STAGE_TOOLS,
        command: CommandSpecV1 {
            template: comparison_command_for_stage(stage_id, &comparison_artifact_ids)?,
        },
        image: ContainerImageRefV1 { image: "bijux-dna-compare".to_string(), digest: None },
        resources: ToolConstraints::default(),
        io: StageIO { inputs: comparison_inputs, outputs: comparison_outputs },
        out_dir: compare_out_dir,
        aux_images: BTreeMap::new(),
        expected_artifact_ids: comparison_artifact_ids.into_iter().map(ArtifactId::new).collect(),
        metrics_schema_ids: Vec::new(),
    }))
}

fn stage_benchmark_comparison_edges(
    steps: &[ExecutionStep],
    compare_step_id: &StepId,
) -> Vec<ExecutionEdge> {
    if !steps.iter().any(|step| &step.step_id == compare_step_id) {
        return Vec::new();
    }
    steps
        .iter()
        .filter(|step| &step.step_id != compare_step_id)
        .map(|step| ExecutionEdge::new(step.step_id.clone(), compare_step_id.clone()))
        .collect()
}

fn normalize_stage_bindings(
    config: &FastqPlanConfig,
) -> Result<(PipelineSpec, Vec<FastqStageBinding>)> {
    if !config.stage_bindings.is_empty() {
        return normalize_explicit_stage_bindings(config);
    }

    if !config.stage_toolsets.is_empty() {
        return normalize_stage_toolsets(config);
    }

    Err(anyhow!(
        "FastqPlanConfig requires a graph-backed planning surface via stage_bindings or stage_toolsets"
    ))
}

fn normalize_explicit_stage_bindings(
    config: &FastqPlanConfig,
) -> Result<(PipelineSpec, Vec<FastqStageBinding>)> {
    if !config.stage_toolsets.is_empty() {
        return Err(ambiguous_graph_surface_error());
    }
    ensure_unique_stage_binding_nodes(&config.stage_bindings)?;
    let pipeline_spec = config
        .pipeline_spec
        .clone()
        .map_or_else(|| implicit_pipeline_spec_from_bindings(&config.stage_bindings), Ok)?;
    Ok((pipeline_spec, config.stage_bindings.clone()))
}

fn normalize_stage_toolsets(
    config: &FastqPlanConfig,
) -> Result<(PipelineSpec, Vec<FastqStageBinding>)> {
    if !config.stage_bindings.is_empty() {
        return Err(ambiguous_graph_surface_error());
    }
    let base_pipeline = config
        .pipeline_spec
        .clone()
        .map_or_else(|| implicit_pipeline_spec_from_toolsets(&config.stage_toolsets), Ok)?;
    let toolsets = toolset_selections_from_bindings(&config.stage_toolsets)?;
    if config.stage_toolsets.iter().all(|binding| binding.tools.len() == 1) {
        let stage_bindings = single_tool_stage_bindings(&config.stage_toolsets);
        ensure_unique_stage_binding_nodes(&stage_bindings)?;
        return Ok((base_pipeline, stage_bindings));
    }
    normalize_expanded_stage_toolsets(config, &base_pipeline, &toolsets)
}

fn ambiguous_graph_surface_error() -> anyhow::Error {
    anyhow!(
        "FastqPlanConfig must use exactly one graph planning surface: stage_bindings or stage_toolsets"
    )
}

fn toolset_selections_from_bindings(
    bindings: &[FastqStageToolsetBinding],
) -> Result<Vec<ToolsetSelection>> {
    bindings
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
                tool_ids: binding.tools.iter().map(|tool| tool.tool_id.to_string()).collect(),
                reason: binding.reason.clone().unwrap_or_default(),
            })
        })
        .collect()
}

fn single_tool_stage_bindings(bindings: &[FastqStageToolsetBinding]) -> Vec<FastqStageBinding> {
    bindings
        .iter()
        .map(|binding| FastqStageBinding {
            stage_id: binding.stage_id.clone(),
            stage_instance_id: binding.stage_instance_id.clone(),
            tool: binding.tools[0].clone(),
            reason: binding.reason.clone(),
            params: binding.params.clone(),
        })
        .collect()
}

fn normalize_expanded_stage_toolsets(
    config: &FastqPlanConfig,
    base_pipeline: &PipelineSpec,
    toolsets: &[ToolsetSelection],
) -> Result<(PipelineSpec, Vec<FastqStageBinding>)> {
    let (expanded_pipeline, expanded_stage_tools) =
        expand_pipeline_stage_tool_routes(base_pipeline, toolsets)?;
    let stage_bindings = expanded_stage_tools
        .into_iter()
        .map(|selection| expanded_stage_binding(config, selection))
        .collect::<Result<Vec<_>>>()?;
    ensure_unique_stage_binding_nodes(&stage_bindings)?;
    Ok((expanded_pipeline, stage_bindings))
}

fn expanded_stage_binding(
    config: &FastqPlanConfig,
    selection: StageToolSelection,
) -> Result<FastqStageBinding> {
    let toolset = source_toolset_for_expanded_selection(
        &config.stage_toolsets,
        &selection.stage_id,
        selection.stage_instance_id.as_deref(),
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
        if node_id_by_stage_id.insert(node.stage_id.clone(), stage_node_id).is_some() {
            return Err(anyhow!(
                "{surface_name} with repeated stage_id {} requires an explicit pipeline_spec",
                node.stage_id
            ));
        }
    }

    let stage_graph = preprocess_pipeline_graph_for_stage_order(
        &nodes.iter().map(|node| StageId::new(node.stage_id.clone())).collect::<Vec<_>>(),
    );
    let edges = stage_graph
        .edges
        .into_iter()
        .map(|edge| PipelineEdgeSpec {
            from: node_id_by_stage_id.get(&edge.from).cloned().unwrap_or(edge.from),
            to: node_id_by_stage_id.get(&edge.to).cloned().unwrap_or(edge.to),
            from_output_id: edge.from_output_id,
            to_input_id: edge.to_input_id,
        })
        .collect();
    Ok(PipelineSpec::graph(nodes, edges))
}

fn base_stage_instance_id(stage_instance_id: Option<&str>) -> Option<&str> {
    stage_instance_id.and_then(|value| value.split(".route.").next())
}

fn source_toolset_for_expanded_selection<'a>(
    toolsets: &'a [FastqStageToolsetBinding],
    stage_id: &str,
    expanded_stage_instance_id: Option<&str>,
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

/// # Errors
/// Returns an error if stage planning fails before graph materialization.
#[allow(non_snake_case)]
pub fn plan_fastq_to_fastq__default__v1_stage_plans(
    inputs: &FastqPipelineInputs,
    options: DefaultPipelineOptions,
) -> Result<Vec<StagePlanV1>> {
    let pipeline = default_pipeline_spec(options);
    let stage_toolsets = stage_toolsets_for_pipeline_nodes(&pipeline, &inputs.stage_toolsets)?;
    let config = FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__default__v1".to_string(),
        policy: inputs.policy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: Some(pipeline),
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
    FastqPlanner::plan_stage_plans(&config)
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
            image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["echo".to_string(), tool_id.to_string()] },
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
        assert_eq!(toolsets[0].stage_instance_id.as_deref(), Some("fastq.validate_reads.first"));
        assert_eq!(toolsets[1].stage_instance_id.as_deref(), Some("fastq.trim_reads.fastp_branch"));
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

        assert!(error.to_string().contains("pipeline nodes/toolset length mismatch"));
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

        assert!(error.to_string().contains("requires an explicit pipeline_spec"));
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
    let steps = stages
        .into_iter()
        .map(|stage| bijux_dna_stage_contract::execution_step_from_stage_plan(&stage))
        .collect();
    let graph = ExecutionGraph::new(
        "fastq-to-bam__default__v1",
        PLANNER_VERSION,
        policy,
        steps,
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
            BamStage::MappingSummary.as_str().to_string(),
            BamStage::Coverage.as_str().to_string(),
            BamStage::Damage.as_str().to_string(),
        ],
        _ => Vec::new(),
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
/// # Errors
/// Returns an error if any stage binding cannot be composed into a governed stage plan.
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
    explicit_stage_inputs: Option<&crate::compose::StageArtifactInputPolicy>,
    out_dir_for_stage: F,
) -> Result<Vec<bijux_dna_stage_contract::StagePlanV1>>
where
    F: FnMut(&FastqStageBinding, &std::path::Path, Option<&std::path::Path>) -> Result<PathBuf>,
{
    compose::compose_fastq_stage_bindings(
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
