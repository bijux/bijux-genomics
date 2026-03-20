fn apply_layout_branching(mut stages: Vec<String>, paired: bool) -> Vec<String> {
    if paired {
        return stages;
    }
    // Single-end runs must not schedule paired-only stages.
    stages.retain(|stage| {
        stage != STAGE_MERGE_PAIRS.as_str()
            && stage != STAGE_CORRECT_ERRORS.as_str()
            && stage != STAGE_EXTRACT_UMIS.as_str()
    });
    stages
}

fn estimate_mean_q(path: &std::path::Path, max_records: usize) -> anyhow::Result<f64> {
    let raw = std::fs::read_to_string(path)?;
    let mut total = 0.0;
    let mut count = 0_u64;
    for (idx, line) in raw.lines().enumerate() {
        if idx % 4 == 3 {
            for byte in line.as_bytes() {
                let score = (*byte as i32 - 33).max(0) as f64;
                total += score;
                count += 1;
            }
            if (idx / 4) + 1 >= max_records {
                break;
            }
        }
    }
    if count == 0 {
        return Ok(0.0);
    }
    Ok(total / count as f64)
}

#[derive(Debug, Clone)]
pub struct FastqPlanConfig {
    pub pipeline_id: String,
    pub policy: PlanPolicy,
    pub pipeline_spec: Option<PipelineSpec>,
    pub stage_bindings: Vec<FastqStageBinding>,
    pub stages: Vec<String>,
    pub tools: Vec<ToolExecutionSpecV1>,
    pub aux_images: BTreeMap<String, ContainerImageRefV1>,
    pub adapter_bank: Option<serde_json::Value>,
    pub polyx_bank: Option<serde_json::Value>,
    pub contaminant_bank: Option<serde_json::Value>,
    pub enable_contaminant_removal: bool,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub reference_fasta: Option<PathBuf>,
    pub out_dir: PathBuf,
    pub tool_reasons: Option<Vec<PlanDecisionReason>>,
    pub allow_planned: bool,
}

#[derive(Debug, Clone)]
pub struct FastqStageBinding {
    pub stage_id: String,
    pub stage_instance_id: Option<String>,
    pub tool: ToolExecutionSpecV1,
    pub reason: Option<PlanDecisionReason>,
    pub params: Option<FastqStageParameters>,
}

#[derive(Debug, Clone)]
pub enum FastqStageParameters {
    TrimTerminalDamage(TrimTerminalDamageStageParams),
    DepleteRrna(DepleteRrnaStageParams),
    DepleteHost(DepleteHostStageParams),
    DepleteReferenceContaminants(DepleteReferenceContaminantsStageParams),
}

#[derive(Debug, Clone)]
pub struct TrimTerminalDamageStageParams {
    pub damage_mode: String,
    pub trim_5p_bases: u32,
    pub trim_3p_bases: u32,
}

impl Default for TrimTerminalDamageStageParams {
    fn default() -> Self {
        Self {
            damage_mode: "ancient".to_string(),
            trim_5p_bases: 2,
            trim_3p_bases: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepleteRrnaStageParams {
    pub rrna_db: String,
    pub min_identity: f64,
}

impl Default for DepleteRrnaStageParams {
    fn default() -> Self {
        Self {
            rrna_db: "rrna_reference".to_string(),
            min_identity: 0.95,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepleteHostStageParams {
    pub host_identity_threshold: f64,
    pub retain_unmapped_only: bool,
}

impl Default for DepleteHostStageParams {
    fn default() -> Self {
        Self {
            host_identity_threshold: 0.95,
            retain_unmapped_only: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepleteReferenceContaminantsStageParams {
    pub decoy_mode: String,
}

impl Default for DepleteReferenceContaminantsStageParams {
    fn default() -> Self {
        Self {
            decoy_mode: "phix_and_spikeins".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FastqStageBenchmarkConfig {
    pub pipeline_id: String,
    pub policy: PlanPolicy,
    pub stage_id: String,
    pub tools: Vec<ToolExecutionSpecV1>,
    pub aux_images: BTreeMap<String, ContainerImageRefV1>,
    pub adapter_bank: Option<serde_json::Value>,
    pub polyx_bank: Option<serde_json::Value>,
    pub contaminant_bank: Option<serde_json::Value>,
    pub enable_contaminant_removal: bool,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub reference_fasta: Option<PathBuf>,
    pub out_dir: PathBuf,
    pub allow_planned: bool,
}

pub struct FastqPlanner;

impl FastqPlanner {
    /// # Errors
    /// Returns an error if planning fails or the plan lint fails.
    pub fn plan(config: &FastqPlanConfig) -> Result<ExecutionGraph> {
        let stage_bindings = normalize_stage_bindings(config)?;
        for binding in &stage_bindings {
            enforce_stage_status(&binding.stage_id, config.allow_planned)?;
        }
        let out_dir = config.out_dir.clone();
        let plans = compose_fastq_stage_bindings(
            &stage_bindings,
            &config.aux_images,
            config.adapter_bank.as_ref(),
            config.polyx_bank.as_ref(),
            config.contaminant_bank.as_ref(),
            config.enable_contaminant_removal,
            &config.r1,
            config.r2.as_deref(),
            config.reference_fasta.as_deref(),
            |binding, _r1, _r2| {
                let stage_dir = binding.stage_id.trim_start_matches(STAGE_PREFIX);
                Ok(out_dir.join(stage_dir).join(binding.tool.tool_id.as_str()))
            },
        )?;
        let edges = execution_edges_for_stage_plans(config.pipeline_spec.as_ref(), &plans)?;
        let graph = ExecutionGraph::new(
            config.pipeline_id.clone(),
            PLANNER_VERSION,
            config.policy,
            plans
                .iter()
                .map(bijux_dna_stage_contract::execution_step_from_stage_plan)
                .collect(),
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
    pub fn plan_stage_benchmark_cohort(config: &FastqStageBenchmarkConfig) -> Result<ExecutionGraph> {
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
                .ok_or_else(|| anyhow!(
                    "missing stage-tool maturity for {} / {}",
                    stage_id.as_str(),
                    tool.tool_id.as_str()
                ))?;
            if maturity == crate::stage_api::StageToolMaturityLevel::PlannedBinding
                && !config.allow_planned
            {
                return Err(anyhow!(
                    "{} is a planned-only binding for {}; rerun with allow_planned to fan out planned tools",
                    tool.tool_id.as_str(),
                    stage_id.as_str()
                ));
            }
            let stage_plans = compose_fastq_pipeline_steps(
                &[config.stage_id.clone()],
                std::slice::from_ref(tool),
                &config.aux_images,
                None,
                config.adapter_bank.as_ref(),
                config.polyx_bank.as_ref(),
                config.contaminant_bank.as_ref(),
                config.enable_contaminant_removal,
                &config.r1,
                config.r2.as_deref(),
                config.reference_fasta.as_deref(),
                |stage, tool, _r1, _r2| {
                    let stage_dir = stage.trim_start_matches(STAGE_PREFIX);
                    Ok(config.out_dir.join(stage_dir).join(tool.tool_id.as_str()))
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
            steps.push(bijux_dna_stage_contract::execution_step_from_stage_plan_with_step_id(
                &plan,
                StepId::new(format!("{}.tool.{}", stage_id.as_str(), tool.tool_id.as_str())),
            ));
        }

        let comparison_artifact_ids =
            bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&stage_id);
        if !comparison_artifact_ids.is_empty() {
            let compare_step_id = StepId::new(format!("{}.compare", stage_id.as_str()));
            let compare_out_dir = config.out_dir.join(stage_id.as_str().trim_start_matches(STAGE_PREFIX)).join("compare");
            let comparison_outputs = comparison_artifact_ids
                .iter()
                .map(|artifact_id| {
                    ArtifactRef::required(
                        ArtifactId::new((*artifact_id).to_string()),
                        compare_out_dir.join(comparison_artifact_file_name(artifact_id)),
                        ArtifactRole::SummaryJson,
                    )
                })
                .collect::<Vec<_>>();
            steps.push(ExecutionStep {
                step_id: compare_step_id,
                stage_id: crate::STAGE_COMPARE_STAGE_TOOLS,
                command: CommandSpecV1 {
                    template: vec![
                        "stage-tool-compare".to_string(),
                        "--stage".to_string(),
                        stage_id.as_str().to_string(),
                    ],
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
                    .map(|artifact_id| ArtifactId::new((*artifact_id).to_string()))
                    .collect(),
                metrics_schema_ids: Vec::new(),
            });
        }

        let compare_step_id = StepId::new(format!("{}.compare", stage_id.as_str()));
        let edges = if steps.iter().any(|step| step.step_id == compare_step_id) {
            steps.iter()
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

fn comparison_artifact_file_name(artifact_id: &str) -> &'static str {
    match artifact_id {
        "trim_tool_benchmark_cohort_json" => "trim_tool_benchmark_cohort.json",
        "trim_tool_comparison_json" => "trim_tool_comparison.json",
        "trim_tool_normalization_json" => "trim_tool_normalization.json",
        "filter_tool_benchmark_cohort_json" => "filter_tool_benchmark_cohort.json",
        "filter_tool_comparison_json" => "filter_tool_comparison.json",
        "filter_tool_normalization_json" => "filter_tool_normalization.json",
        "merge_tool_benchmark_cohort_json" => "merge_tool_benchmark_cohort.json",
        "merge_tool_comparison_json" => "merge_tool_comparison.json",
        "merge_tool_normalization_json" => "merge_tool_normalization.json",
        "low_complexity_tool_benchmark_cohort_json" => "low_complexity_tool_benchmark_cohort.json",
        "low_complexity_tool_comparison_json" => "low_complexity_tool_comparison.json",
        "low_complexity_tool_normalization_json" => "low_complexity_tool_normalization.json",
        "dedup_tool_benchmark_cohort_json" => "dedup_tool_benchmark_cohort.json",
        "dedup_tool_comparison_json" => "dedup_tool_comparison.json",
        "dedup_tool_normalization_json" => "dedup_tool_normalization.json",
        "read_length_tool_benchmark_cohort_json" => "read_length_tool_benchmark_cohort.json",
        "read_length_tool_comparison_json" => "read_length_tool_comparison.json",
        "read_length_tool_normalization_json" => "read_length_tool_normalization.json",
        "correction_tool_benchmark_cohort_json" => "correction_tool_benchmark_cohort.json",
        "correction_tool_comparison_json" => "correction_tool_comparison.json",
        "correction_tool_normalization_json" => "correction_tool_normalization.json",
        "taxonomy_tool_benchmark_cohort_json" => "taxonomy_tool_benchmark_cohort.json",
        "taxonomy_tool_comparison_json" => "taxonomy_tool_comparison.json",
        "taxonomy_tool_normalization_json" => "taxonomy_tool_normalization.json",
        _ => "comparison.json",
    }
}

fn normalize_stage_bindings(config: &FastqPlanConfig) -> Result<Vec<FastqStageBinding>> {
    if !config.stage_bindings.is_empty() {
        if !config.stages.is_empty() || !config.tools.is_empty() || config.tool_reasons.is_some() {
            return Err(anyhow!(
                "FastqPlanConfig must use either stage_bindings or legacy stages/tools fields, not both"
            ));
        }
        ensure_unique_stage_binding_nodes(&config.stage_bindings)?;
        return Ok(config.stage_bindings.clone());
    }

    if config.stages.len() != config.tools.len() {
        return Err(anyhow!(
            "pipeline stages/tools length mismatch: {} vs {}",
            config.stages.len(),
            config.tools.len()
        ));
    }
    if let Some(reasons) = config.tool_reasons.as_ref() {
        if reasons.len() != config.stages.len() {
            return Err(anyhow!(
                "pipeline stages/tool_reasons length mismatch: {} vs {}",
                config.stages.len(),
                reasons.len()
            ));
        }
    }

    let bindings = config
        .stages
        .iter()
        .zip(config.tools.iter())
        .enumerate()
        .map(|(idx, (stage_id, tool))| FastqStageBinding {
            stage_id: stage_id.clone(),
            stage_instance_id: None,
            tool: tool.clone(),
            reason: config
                .tool_reasons
                .as_ref()
                .and_then(|reasons| reasons.get(idx).cloned()),
            params: None,
        })
        .collect::<Vec<_>>();
    ensure_unique_stage_binding_nodes(&bindings)?;
    Ok(bindings)
}

fn execution_edges_for_stage_plans(
    pipeline_spec: Option<&PipelineSpec>,
    plans: &[StagePlanV1],
) -> Result<Vec<ExecutionEdge>> {
    let Some(pipeline_spec) = pipeline_spec.filter(|spec| spec.declares_graph_topology()) else {
        return Ok(default_edges_for_stages(plans)
            .into_iter()
            .map(|edge| {
                ExecutionEdge::new(
                    StepId::new(edge.from().to_string()),
                    StepId::new(edge.to().to_string()),
                )
            })
            .collect());
    };

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

    pipeline_spec
        .edges
        .iter()
        .map(|edge| execution_edge_from_pipeline_edge(edge, &plan_nodes))
        .collect()
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
    pub tools: Vec<ToolExecutionSpecV1>,
    pub aux_images: BTreeMap<String, ContainerImageRefV1>,
    pub adapter_bank: Option<serde_json::Value>,
    pub polyx_bank: Option<serde_json::Value>,
    pub contaminant_bank: Option<serde_json::Value>,
    pub enable_contaminant_removal: bool,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub reference_fasta: Option<PathBuf>,
    pub out_dir: PathBuf,
    pub tool_reasons: Option<Vec<PlanDecisionReason>>,
}

/// # Errors
/// Returns an error if planning fails.
#[allow(non_snake_case)]
pub fn plan_fastq_to_fastq__default__v1(
    inputs: &FastqPipelineInputs,
    options: DefaultPipelineOptions,
) -> Result<ExecutionGraph> {
    let pipeline = default_pipeline_spec(options);
    let config = FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__default__v1".to_string(),
        policy: inputs.policy,
        pipeline_spec: Some(pipeline.clone()),
        stage_bindings: Vec::new(),
        stages: pipeline.stages,
        tools: inputs.tools.clone(),
        aux_images: inputs.aux_images.clone(),
        adapter_bank: inputs.adapter_bank.clone(),
        polyx_bank: inputs.polyx_bank.clone(),
        contaminant_bank: inputs.contaminant_bank.clone(),
        enable_contaminant_removal: inputs.enable_contaminant_removal,
        r1: inputs.r1.clone(),
        r2: inputs.r2.clone(),
        reference_fasta: inputs.reference_fasta.clone(),
        out_dir: inputs.out_dir.clone(),
        tool_reasons: inputs.tool_reasons.clone(),
        allow_planned: false,
    };
    FastqPlanner::plan(&config)
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
pub fn compose_fastq_pipeline_steps<F>(
    stages: &[String],
    tools: &[ToolExecutionSpecV1],
    aux_images: &BTreeMap<String, ContainerImageRefV1>,
    tool_reasons: Option<&[PlanDecisionReason]>,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    enable_contaminant_removal: bool,
    r1: &std::path::Path,
    r2: Option<&std::path::Path>,
    reference_fasta: Option<&std::path::Path>,
    mut out_dir_for_stage: F,
) -> Result<Vec<bijux_dna_stage_contract::StagePlanV1>>
where
    F: FnMut(
        &str,
        &ToolExecutionSpecV1,
        &std::path::Path,
        Option<&std::path::Path>,
    ) -> Result<PathBuf>,
{
    let stage_bindings = stages
        .iter()
        .zip(tools.iter())
        .enumerate()
        .map(|(idx, (stage_id, tool))| FastqStageBinding {
            stage_id: stage_id.clone(),
            stage_instance_id: None,
            tool: tool.clone(),
            reason: tool_reasons.and_then(|reasons| reasons.get(idx).cloned()),
            params: None,
        })
        .collect::<Vec<_>>();
    compose_fastq_stage_bindings(
        &stage_bindings,
        aux_images,
        adapter_bank,
        polyx_bank,
        contaminant_bank,
        enable_contaminant_removal,
        r1,
        r2,
        reference_fasta,
        |binding, current_r1, current_r2| {
            out_dir_for_stage(&binding.stage_id, &binding.tool, current_r1, current_r2)
        },
    )
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
    out_dir_for_stage: F,
) -> Result<Vec<bijux_dna_stage_contract::StagePlanV1>>
where
    F: FnMut(
        &FastqStageBinding,
        &std::path::Path,
        Option<&std::path::Path>,
    ) -> Result<PathBuf>,
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
        out_dir_for_stage,
    )
}

#[derive(Debug, Clone)]
pub struct ToolSelection {
    pub tool_id: String,
    pub reason: PlanDecisionReason,
}

#[derive(Debug, Clone)]
pub struct ToolsetSelection {
    pub stage_id: String,
    pub tool_ids: Vec<String>,
    pub reason: PlanDecisionReason,
}

/// # Errors
/// Returns an error if toolset selection fails.
pub fn select_preprocess_toolsets(
    pipeline: &PipelineSpec,
    mode: crate::stage_api::ToolsetExecutionMode,
    allow_planned: bool,
) -> Result<Vec<ToolsetSelection>> {
    let mut selections = Vec::new();
    for stage in &pipeline.stages {
        enforce_stage_status(stage, allow_planned)?;
        let stage_id = StageId::new(stage.clone());
        let tool_ids = crate::stage_api::toolset_for_stage(&stage_id, mode)
            .into_iter()
            .map(|tool_id| tool_id.to_string())
            .collect::<Vec<_>>();
        selections.push(ToolsetSelection {
            stage_id: stage.clone(),
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

/// # Errors
/// Returns an error if tool selection fails.
pub fn select_preprocess_tools(
    registry: &bijux_dna_core::contract::ToolRegistry,
    pipeline: &PipelineSpec,
    args: &crate::selection::args::BenchFastqPreprocessArgs,
    bench_repo: Option<&dyn BenchResultsRepository>,
) -> Result<Vec<ToolSelection>> {
    let mut selected_tools: Vec<ToolSelection> = pipeline
        .stages
        .iter()
        .map(|stage| {
            let stage_id = StageId::new(stage.clone());
            let tool_id = crate::selection::default_tool_for_stage(&stage_id)
                .map(|tool| tool.to_string())
                .or_else(|| {
                    registry
                        .tools_for_stage(&stage_id)
                        .first()
                        .map(|tool| tool.tool_id.to_string())
                })
                .ok_or_else(|| anyhow!("no default tool for stage {stage}"))?;
            Ok(ToolSelection {
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
        for stage in &pipeline.stages {
            let stage_id = bijux_dna_core::ids::StageId::new(stage.clone());
            let tool_ids: Vec<String> = registry
                .tools_for_stage(&stage_id)
                .iter()
                .map(|tool| tool.tool_id.to_string())
                .collect();
            let mut tool_records = Vec::new();
            for tool in &tool_ids {
                let records = repo.bench_results(&stage_id, tool, &corpus)?;
                tool_records.push((tool.clone(), records));
            }
            let selection = bijux_dna_core::contract::select_stage(
                &stage_id,
                &tool_records,
                &objective,
                args.allow_partial,
            );
            selections.push(selection);
        }
        for (idx, selection) in selections.into_iter().enumerate() {
            if let Some(selected) = selection.selected {
                selected_tools[idx] = ToolSelection {
                    tool_id: selected,
                    reason: PlanDecisionReason::new(
                        PlanReasonKind::InputAssessed,
                        "auto-selected from benchmark corpus",
                    ),
                };
            }
        }
    }

    Ok(selected_tools)
}
include!("tool_selection_facade.rs");

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
mod unit_checks;
