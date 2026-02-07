use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_core::contract::PipelineSpec;
use bijux_core::contract::PlanPolicy;
use bijux_core::contract::{ExecutionEdge, ExecutionGraph};
use bijux_core::prelude::input_assessment::{assess_input_dir, FastqLayout};
use bijux_core::prelude::{ContainerImageRefV1, StageId, StepId, ToolExecutionSpecV1};
use bijux_domain_bam::BamStage;
use bijux_domain_fastq::{assess_merge_suitability, canonical_stage_order};
use bijux_domain_fastq::{
    STAGE_CORRECT, STAGE_DETECT_ADAPTERS, STAGE_FILTER, STAGE_MERGE, STAGE_PREFIX,
    STAGE_PREPROCESS, STAGE_QC_POST, STAGE_SCREEN, STAGE_STATS_NEUTRAL, STAGE_TRIM, STAGE_UMI,
    STAGE_VALIDATE_PRE,
};
use bijux_pipelines::STAGE_CORE_PREPARE_REFERENCE;
use bijux_stage_contract::{
    default_edges_for_stages, PlanDecisionReason, PlanReasonKind, StagePlanV1,
};

pub const PLANNER_VERSION: &str = "bijux-planner-fastq.v1";
pub const TOOL_SEQKIT: &str = "seqkit";
pub const STAGE_REPORT_AGGREGATE: StageId = StageId::from_static("report.aggregate");

pub use bijux_domain_fastq::BenchResultsRepository;

mod report_stage;
mod selection;
pub mod tool_adapters;

pub use report_stage::report_stage_step;
pub use selection::args;

pub mod stage_api;

fn required_stage_ids() -> Vec<String> {
    let mut stages: Vec<String> = canonical_stage_order()
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .collect();
    if !stages.iter().any(|stage| stage == STAGE_QC_POST.as_str()) {
        stages.push(STAGE_QC_POST.as_str().to_string());
    }
    stages
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct DefaultPipelineOptions {
    pub paired: bool,
    pub enable_merge: bool,
    pub enable_correct: bool,
    pub enable_qc_post: bool,
    pub enable_screen: bool,
}

impl Default for DefaultPipelineOptions {
    fn default() -> Self {
        Self {
            paired: false,
            enable_merge: true,
            enable_correct: false,
            enable_qc_post: true,
            enable_screen: false,
        }
    }
}

#[must_use]
pub fn default_pipeline_spec(options: DefaultPipelineOptions) -> PipelineSpec {
    let mut stages = required_stage_ids();
    if options.paired && options.enable_correct {
        stages.push(STAGE_CORRECT.as_str().to_string());
    }
    if options.paired && options.enable_merge {
        stages.push(STAGE_MERGE.as_str().to_string());
    }
    if options.enable_screen && !stages.iter().any(|stage| stage == STAGE_SCREEN.as_str()) {
        stages.push(STAGE_SCREEN.as_str().to_string());
    }
    if options.enable_qc_post && !stages.iter().any(|stage| stage == STAGE_QC_POST.as_str()) {
        stages.push(STAGE_QC_POST.as_str().to_string());
    }
    PipelineSpec { stages }
}

#[derive(Debug, Clone)]
pub struct PreprocessPolicyDecision {
    pub adapter_inference: Option<serde_json::Value>,
    pub adapter_bank_preset_override: Option<String>,
    pub pipeline_stages: Vec<String>,
    pub pipeline_tools: Vec<String>,
    pub stage_skips: Vec<serde_json::Value>,
}

#[must_use]
pub fn apply_preprocess_policy(
    pipeline_stages: Vec<String>,
    pipeline_tools: Vec<String>,
) -> PreprocessPolicyDecision {
    let mut pipeline_stages = pipeline_stages;
    let mut pipeline_tools = pipeline_tools;
    let mut stage_skips = Vec::new();

    if let (Some(trim_idx), Some(filter_idx)) = (
        pipeline_stages
            .iter()
            .position(|stage| stage == STAGE_TRIM.as_str()),
        pipeline_stages
            .iter()
            .position(|stage| stage == STAGE_FILTER.as_str()),
    ) {
        let trim_tool = pipeline_tools.get(trim_idx).map(String::as_str);
        let filter_tool = pipeline_tools.get(filter_idx).map(String::as_str);
        if trim_tool == Some("fastp") && filter_tool == Some("fastp") {
            let skipped_stage = pipeline_stages.remove(filter_idx);
            let skipped_tool = pipeline_tools.remove(filter_idx);
            stage_skips.push(serde_json::json!({
                "stage_id": skipped_stage,
                "tool_id": skipped_tool,
                "reason": "fastp trimming already performs quality filtering; filter stage skipped",
                "equivalent_params": {
                    "quality_filtering": true
                }
            }));
        }
    }

    PreprocessPolicyDecision {
        adapter_inference: None,
        adapter_bank_preset_override: None,
        pipeline_stages,
        pipeline_tools,
        stage_skips,
    }
}

#[derive(Debug, Clone)]
pub struct PreprocessDecisions {
    pub enable_merge: bool,
    pub enable_correct: bool,
    pub merge_decision: Option<MergeDecisionTrace>,
    pub correct_decision: Option<CorrectDecisionTrace>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeDecisionTrace {
    pub enabled: bool,
    pub suitable: bool,
    pub forced: bool,
    pub reason: String,
    pub r1_mean_len: Option<usize>,
    pub r2_mean_len: Option<usize>,
    pub predicted_merge_rate: Option<f64>,
    pub probe_pairs: Option<usize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorrectDecisionTrace {
    pub enabled: bool,
    pub auto_enabled: bool,
    pub reason: String,
    pub mean_q_estimate: Option<f64>,
}

#[must_use]
pub fn preprocess_decisions(
    args: &crate::selection::args::BenchFastqPreprocessArgs,
) -> PreprocessDecisions {
    let mut merge_decision = None;
    let enable_merge = if let Some(r2) = args.r2.as_ref() {
        if args.force_merge {
            merge_decision = Some(MergeDecisionTrace {
                enabled: true,
                suitable: true,
                forced: true,
                reason: "merge forced by flag".to_string(),
                r1_mean_len: None,
                r2_mean_len: None,
                predicted_merge_rate: None,
                probe_pairs: None,
            });
            true
        } else {
            match assess_merge_suitability(&args.r1, r2) {
                Ok(suitability) => {
                    let enabled = suitability.suitable;
                    merge_decision = Some(MergeDecisionTrace {
                        enabled,
                        suitable: suitability.suitable,
                        forced: false,
                        reason: suitability.reason,
                        r1_mean_len: suitability.r1_mean_len,
                        r2_mean_len: suitability.r2_mean_len,
                        predicted_merge_rate: suitability.predicted_merge_rate,
                        probe_pairs: suitability.probe_pairs,
                    });
                    enabled
                }
                Err(err) => {
                    merge_decision = Some(MergeDecisionTrace {
                        enabled: false,
                        suitable: false,
                        forced: false,
                        reason: format!("merge suitability check failed: {err}"),
                        r1_mean_len: None,
                        r2_mean_len: None,
                        predicted_merge_rate: None,
                        probe_pairs: None,
                    });
                    false
                }
            }
        }
    } else {
        false
    };
    let mut enable_merge = enable_merge;
    if enable_merge {
        if let Some(parent) = args.r1.parent() {
            if let Ok(assessment) = assess_input_dir(parent) {
                let paired = assessment
                    .samples
                    .iter()
                    .any(|sample| sample.id.layout == FastqLayout::PairedEnd);
                if !paired {
                    enable_merge = false;
                    merge_decision = Some(MergeDecisionTrace {
                        enabled: false,
                        suitable: false,
                        forced: false,
                        reason: "input assessment indicates single-end reads".to_string(),
                        r1_mean_len: None,
                        r2_mean_len: None,
                        predicted_merge_rate: None,
                        probe_pairs: None,
                    });
                }
            }
        }
    }

    let mut correct_decision = None;
    let mut enable_correct = args.enable_correct;
    if !enable_correct && args.r2.is_some() {
        let thresholds = bijux_domain_fastq::thresholds_from_env();
        if let Ok(mean_q) = estimate_mean_q(&args.r1, 256) {
            if mean_q < thresholds.mean_q_warn {
                enable_correct = true;
                correct_decision = Some(CorrectDecisionTrace {
                    enabled: true,
                    auto_enabled: true,
                    reason: format!(
                        "mean_q estimate {:.2} below warn threshold {:.2}",
                        mean_q, thresholds.mean_q_warn
                    ),
                    mean_q_estimate: Some(mean_q),
                });
            } else {
                correct_decision = Some(CorrectDecisionTrace {
                    enabled: false,
                    auto_enabled: false,
                    reason: "mean_q estimate within expected range".to_string(),
                    mean_q_estimate: Some(mean_q),
                });
            }
        }
    } else if enable_correct {
        correct_decision = Some(CorrectDecisionTrace {
            enabled: true,
            auto_enabled: false,
            reason: "error correction enabled by user flag".to_string(),
            mean_q_estimate: None,
        });
    }
    PreprocessDecisions {
        enable_merge,
        enable_correct,
        merge_decision,
        correct_decision,
    }
}

#[must_use]
pub fn plan_preprocess(
    args: &crate::selection::args::BenchFastqPreprocessArgs,
    pipeline: PipelineSpec,
) -> crate::tool_adapters::fastq::preprocess::PreprocessPlan {
    crate::tool_adapters::fastq::preprocess::PreprocessPlan {
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        stages: pipeline.stages,
        enable_contaminant_removal: args.enable_contaminant_removal,
    }
}

#[must_use]
pub fn resolve_preprocess_pipeline(
    args: &crate::selection::args::BenchFastqPreprocessArgs,
    decisions: &PreprocessDecisions,
) -> PipelineSpec {
    let enable_merge = decisions.enable_merge;
    let enable_correct = decisions.enable_correct;
    let enable_qc_post = !args.no_qc_post;
    let enable_screen = args.contaminant_preset.is_some();
    if let Some(profile_id) = args.profile.as_deref() {
        match bijux_pipelines::registry::profile_by_id(bijux_pipelines::Domain::Fastq, profile_id) {
            Ok(profile) => {
                let mut stages: Vec<String> = fastq_pipeline_stage_ids(profile.id.as_str());
                if !enable_merge {
                    stages.retain(|stage| stage != STAGE_MERGE.as_str());
                }
                if !enable_correct {
                    stages.retain(|stage| stage != STAGE_CORRECT.as_str());
                }
                if !enable_qc_post {
                    stages.retain(|stage| stage != STAGE_QC_POST.as_str());
                }
                if !enable_screen {
                    stages.retain(|stage| stage != STAGE_SCREEN.as_str());
                }
                PipelineSpec { stages }
            }
            Err(err) => {
                eprintln!("unknown fastq profile {profile_id}: {err}; using default pipeline");
                default_pipeline_spec(DefaultPipelineOptions {
                    paired: args.r2.is_some(),
                    enable_merge,
                    enable_correct,
                    enable_qc_post,
                    enable_screen,
                })
            }
        }
    } else {
        default_pipeline_spec(DefaultPipelineOptions {
            paired: args.r2.is_some(),
            enable_merge,
            enable_correct,
            enable_qc_post,
            enable_screen,
        })
    }
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
    pub out_dir: PathBuf,
    pub tool_reasons: Option<Vec<PlanDecisionReason>>,
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
                .map(bijux_stage_contract::execution_step_from_stage_plan)
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
        out_dir: inputs.out_dir.clone(),
        tool_reasons: inputs.tool_reasons.clone(),
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
            .map(bijux_stage_contract::execution_step_from_stage_plan)
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
pub fn cross_fastq_to_bam_stage_ids(profile_id: &str) -> Vec<String> {
    match profile_id {
        "fastq-to-bam__adna_shotgun__v1" | "fastq-to-bam__default__v1" => vec![
            STAGE_PREPROCESS.as_str().to_string(),
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
    mut out_dir_for_stage: F,
) -> Result<Vec<bijux_stage_contract::StagePlanV1>>
where
    F: FnMut(
        &str,
        &ToolExecutionSpecV1,
        &std::path::Path,
        Option<&std::path::Path>,
    ) -> Result<PathBuf>,
{
    if stages.len() != tools.len() {
        return Err(anyhow!(
            "pipeline stages/tools length mismatch: {} vs {}",
            stages.len(),
            tools.len()
        ));
    }
    let mut current_r1 = r1.to_path_buf();
    let raw_r1 = r1.to_path_buf();
    let mut current_r2 = r2.map(|path| path.to_path_buf());
    let mut plans = Vec::new();
    for (idx, (stage, tool)) in stages.iter().zip(tools.iter()).enumerate() {
        let out_dir = out_dir_for_stage(stage, tool, &current_r1, current_r2.as_deref())?;
        let stage_id: &str = stage;
        let (plan, next_r1, next_r2) = match stage_id {
            stage if stage == STAGE_DETECT_ADAPTERS.as_str() => {
                let plan =
                    crate::tool_adapters::fastq::detect_adapters::plan(tool, &current_r1, &out_dir);
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_TRIM.as_str() => {
                let plan = crate::tool_adapters::fastq::trim::plan(
                    tool,
                    &current_r1,
                    &out_dir,
                    adapter_bank,
                    polyx_bank,
                    contaminant_bank,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_FILTER.as_str() => {
                let mut filter_options =
                    crate::tool_adapters::fastq::filter::FilterPlanOptions::default();
                if adapter_bank.is_some() {
                    filter_options.redundant_filters.push("adapter".to_string());
                }
                if polyx_bank.is_some() {
                    filter_options.redundant_filters.push("polyx".to_string());
                }
                if enable_contaminant_removal && contaminant_bank.is_some() {
                    filter_options.kmer_ref =
                        crate::tool_adapters::fastq::filter::default_kmer_ref();
                }
                let plan = crate::tool_adapters::fastq::filter::plan_filter(
                    tool,
                    &current_r1,
                    &out_dir,
                    &filter_options,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_VALIDATE_PRE.as_str() => {
                let plan =
                    crate::tool_adapters::fastq::validate_pre::plan(tool, &current_r1, &out_dir);
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_MERGE.as_str() => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow!("merge requires r2"))?;
                let plan = crate::tool_adapters::fastq::merge::plan_merge(
                    tool,
                    &current_r1,
                    r2,
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_CORRECT.as_str() => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow!("correct requires r2"))?;
                let plan = crate::tool_adapters::fastq::correct::plan_correct(
                    tool,
                    &current_r1,
                    r2,
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = plan.io.outputs[1].path.clone();
                (plan, next_r1, Some(next_r2))
            }
            stage if stage == STAGE_UMI.as_str() => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow!("umi requires r2"))?;
                let plan =
                    crate::tool_adapters::fastq::umi::plan_umi(tool, &current_r1, r2, &out_dir)?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = plan.io.outputs[1].path.clone();
                (plan, next_r1, Some(next_r2))
            }
            stage if stage == STAGE_QC_POST.as_str() => {
                let mut stage_aux_images = std::collections::BTreeMap::new();
                if tool.tool_id.0 == "multiqc" {
                    for aux_tool in crate::tool_adapters::fastq::qc_post::aux_tool_ids() {
                        if let Some(image) = aux_images.get(*aux_tool) {
                            stage_aux_images.insert(aux_tool.to_string(), image.clone());
                        }
                    }
                }
                let plan = crate::tool_adapters::fastq::qc_post::plan_qc_post(
                    tool,
                    &current_r1,
                    &out_dir,
                    stage_aux_images,
                    Some(raw_r1.as_path()),
                )?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_SCREEN.as_str() => {
                let plan =
                    crate::tool_adapters::fastq::screen::plan_screen(tool, &current_r1, &out_dir)?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_STATS_NEUTRAL.as_str() => {
                let plan = crate::tool_adapters::fastq::stats_neutral::plan_stats_neutral(
                    tool,
                    &current_r1,
                    &out_dir,
                )?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            _ => {
                return Err(anyhow!("unsupported stage in fastq pipeline: {stage}"));
            }
        };
        let mut plan = plan;
        if let Some(reasons) = tool_reasons {
            if let Some(reason) = reasons.get(idx) {
                plan.reason = reason.clone();
            }
        } else {
            plan.reason = PlanDecisionReason::new(
                PlanReasonKind::Default,
                format!("tool {} selected by planner", plan.tool_id.0),
            );
        }
        plans.push(plan);
        current_r1 = next_r1;
        current_r2 = next_r2;
    }
    Ok(plans)
}

#[derive(Debug, Clone)]
pub struct ToolSelection {
    pub tool_id: String,
    pub reason: PlanDecisionReason,
}

/// # Errors
/// Returns an error if tool selection fails.
pub fn select_preprocess_tools(
    registry: &bijux_core::contract::ToolRegistry,
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
        let corpus = bijux_domain_fastq::bench_corpus(corpus_id);
        let objective = bijux_core::contract::objective_spec(args.objective);
        let repo = bench_repo.ok_or_else(|| {
            anyhow!("bench results repository required for --auto tool selection")
        })?;
        let mut selections = Vec::new();
        for stage in &pipeline.stages {
            let stage_id = bijux_core::ids::StageId::new(stage.clone());
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
            let selection = bijux_core::contract::select_stage(
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

pub fn select_trim_tools(tools: &[String], allow_experimental: bool) -> Result<Vec<String>> {
    let allowed = [
        "fastp",
        "cutadapt",
        "bbduk",
        "adapterremoval",
        "trimmomatic",
        "trim_galore",
        "atropos",
        "seqpurge",
    ];
    let mut allowlist = allowed.to_vec();
    if !allow_experimental {
        allowlist.retain(|tool| *tool != "seqpurge");
    }
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_validate_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "seqtk",
        "fastqc",
        "fastqvalidator",
        "fastqvalidator_official",
        "fqtools",
    ];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_filter_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["prinseq", "fastp", "seqkit"];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_merge_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["pear", "vsearch", "bbmerge", "flash2"];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_correct_tools(tools: &[String], allow_experimental: bool) -> Result<Vec<String>> {
    let allowed = ["rcorrector", "spades", "bayeshammer", "lighter", "musket"];
    let mut allowlist = allowed.to_vec();
    if !allow_experimental {
        allowlist.retain(|tool| *tool == "rcorrector");
    }
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_qc_post_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["fastqc", "multiqc"];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_umi_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["umi_tools"];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_screen_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "kraken2",
        "centrifuge",
        "metaphlan",
        "kaiju",
        "fastq_screen",
    ];
    select_tools_with_allowlist(tools, &allowed)
}

pub fn select_stats_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["seqkit_stats"];
    select_tools_with_allowlist(tools, &allowed)
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

fn select_tools_with_allowlist(tools: &[String], allowlist: &[&str]) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    for tool in &normalized {
        if !allowlist.contains(&tool.as_str()) {
            return Err(anyhow!("unsupported tool: {tool}"));
        }
    }
    Ok(normalized)
}

#[must_use]
pub fn apply_tool_overrides(
    base: BTreeMap<String, String>,
    profile: BTreeMap<String, String>,
    cli_overrides: BTreeMap<String, String>,
    forced_overrides: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut merged = base;
    for (stage, tool) in profile {
        merged.insert(stage, tool);
    }
    for (stage, tool) in cli_overrides {
        merged.insert(stage, tool);
    }
    for (stage, tool) in forced_overrides {
        merged.insert(stage, tool);
    }
    merged
}

#[must_use]
pub fn fastq_pipeline_stage_ids(profile_id: &str) -> Vec<String> {
    match profile_id {
        "fastq-to-fastq__default__v1" | "fastq-to-fastq__minimal__v1" => required_stage_ids(),
        _ => required_stage_ids(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn select_trim_tools_dedup_and_sort() {
        let tools = vec![
            "fastp".to_string(),
            "FASTP".to_string(),
            "cutadapt".to_string(),
        ];
        match select_trim_tools(&tools, false) {
            Ok(normalized) => {
                assert_eq!(
                    normalized,
                    vec!["cutadapt".to_string(), "fastp".to_string()]
                );
            }
            Err(err) => panic!("normalize failed: {err}"),
        }
    }

    #[test]
    fn select_trim_tools_blocks_experimental_by_default() {
        let tools = vec!["seqpurge".to_string()];
        match select_trim_tools(&tools, false) {
            Ok(_) => panic!("expected failure"),
            Err(err) => assert!(err.to_string().contains("unsupported tool")),
        }
    }

    #[test]
    fn select_trim_tools_allows_experimental_when_enabled() {
        let tools = vec!["seqpurge".to_string()];
        match select_trim_tools(&tools, true) {
            Ok(normalized) => assert_eq!(normalized, vec!["seqpurge".to_string()]),
            Err(err) => panic!("normalize failed: {err}"),
        }
    }

    #[test]
    fn select_tools_rejects_empty() {
        match select_validate_tools(&[]) {
            Ok(_) => panic!("expected empty failure"),
            Err(err) => assert!(err.to_string().contains("no tools specified")),
        }
    }
}
