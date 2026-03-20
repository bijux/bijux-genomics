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
        if config.stages.len() != config.tools.len() {
            return Err(anyhow!(
                "pipeline stages/tools length mismatch: {} vs {}",
                config.stages.len(),
                config.tools.len()
            ));
        }
        for stage in &config.stages {
            enforce_stage_status(stage, config.allow_planned)?;
        }
        let out_dir = config.out_dir.clone();
        let plans = compose_fastq_pipeline_steps(
            &config.stages,
            &config.tools,
            &config.aux_images,
            config.tool_reasons.as_deref(),
            config.adapter_bank.as_ref(),
            config.polyx_bank.as_ref(),
            config.contaminant_bank.as_ref(),
            config.enable_contaminant_removal,
            &config.r1,
            config.r2.as_deref(),
            config.reference_fasta.as_deref(),
            |stage, tool, _r1, _r2| {
                let stage_dir = stage.trim_start_matches(STAGE_PREFIX);
                Ok(out_dir.join(stage_dir).join(tool.tool_id.as_str()))
            },
        )?;
        let edges = default_edges_for_stages(&plans);
        let graph = ExecutionGraph::new(
            config.pipeline_id.clone(),
            PLANNER_VERSION,
            config.policy,
            plans
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
            steps.push(bijux_dna_stage_contract::execution_step_from_stage_plan_with_step_id(
                &plan,
                StepId::new(format!("{}.tool.{}", stage_id.as_str(), tool.tool_id.as_str())),
            ));
        }

        Ok(ExecutionGraph::new(
            config.pipeline_id.clone(),
            PLANNER_VERSION,
            config.policy,
            steps,
            Vec::new(),
        )?)
    }
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
    out_dir_for_stage: F,
) -> Result<Vec<bijux_dna_stage_contract::StagePlanV1>>
where
    F: FnMut(
        &str,
        &ToolExecutionSpecV1,
        &std::path::Path,
        Option<&std::path::Path>,
    ) -> Result<PathBuf>,
{
    plan_compose::compose_fastq_pipeline_steps(
        stages,
        tools,
        aux_images,
        tool_reasons,
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
