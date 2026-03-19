use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_dna_core::contract::PipelineSpec;
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph};
use bijux_dna_core::id_catalog;
use bijux_dna_core::prelude::input_assessment::{assess_input_dir, FastqLayout};
use bijux_dna_core::prelude::{ContainerImageRefV1, StageId, StepId, ToolExecutionSpecV1};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_domain_fastq::{assess_merge_suitability, canonical_stage_order};
use bijux_dna_domain_fastq::{
    stages::ids::{STAGE_CONTAMINANT_SCREEN, STAGE_HOST_DEPLETION},
    FastqPipelineMode, STAGE_ABUNDANCE_NORMALIZATION, STAGE_ASV_INFERENCE, STAGE_CHIMERA_DETECTION,
    STAGE_CORRECT, STAGE_DAMAGE_AWARE_PRETRIM, STAGE_DEDUPLICATE, STAGE_DETECT_ADAPTERS,
    STAGE_FILTER, STAGE_LOW_COMPLEXITY, STAGE_MERGE, STAGE_OTU_CLUSTERING, STAGE_PREFIX,
    STAGE_PREPROCESS, STAGE_PRIMER_NORMALIZATION, STAGE_QC_POST, STAGE_RRNA, STAGE_SCREEN,
    STAGE_STATS_NEUTRAL, STAGE_TRIM, STAGE_UMI, STAGE_VALIDATE_PRE,
};
use bijux_dna_pipelines::STAGE_CORE_PREPARE_REFERENCE;
use bijux_dna_stage_contract::{
    default_edges_for_stages, PlanDecisionReason, PlanReasonKind, StagePlanV1,
};

pub const PLANNER_VERSION: &str = "bijux-dna-planner-fastq.v1";
pub const TOOL_SEQKIT: &str = "seqkit";
pub const STAGE_REPORT_AGGREGATE: StageId = StageId::from_static("report.aggregate");

pub use bijux_dna_domain_fastq::BenchResultsRepository;

mod plan_compose;
mod report_stage;
mod selection;
pub mod tool_adapters;

pub use report_stage::report_stage_step;
pub use selection::args;

pub mod stage_api;

fn required_id_catalog() -> Vec<String> {
    let mut stages = bijux_dna_pipelines::fastq::fastq_default_profile()
        .capabilities
        .required_stages
        .iter()
        .map(|stage| (*stage).to_string())
        .collect::<Vec<_>>();
    stages.retain(|stage| stage.starts_with(STAGE_PREFIX));
    let canonical = canonical_stage_order()
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<Vec<_>>();
    stages.retain(|stage| canonical.contains(stage));
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
    pub mode: FastqPipelineMode,
}

impl Default for DefaultPipelineOptions {
    fn default() -> Self {
        Self {
            paired: false,
            enable_merge: true,
            enable_correct: false,
            enable_qc_post: true,
            enable_screen: false,
            mode: FastqPipelineMode::Shotgun,
        }
    }
}

#[must_use]
pub fn default_pipeline_spec(options: DefaultPipelineOptions) -> PipelineSpec {
    let mut stages = if options.mode == FastqPipelineMode::Amplicon {
        vec![
            STAGE_VALIDATE_PRE.as_str().to_string(),
            STAGE_DETECT_ADAPTERS.as_str().to_string(),
            STAGE_DAMAGE_AWARE_PRETRIM.as_str().to_string(),
            STAGE_PRIMER_NORMALIZATION.as_str().to_string(),
            STAGE_TRIM.as_str().to_string(),
            STAGE_FILTER.as_str().to_string(),
            STAGE_CHIMERA_DETECTION.as_str().to_string(),
            STAGE_ASV_INFERENCE.as_str().to_string(),
            STAGE_ABUNDANCE_NORMALIZATION.as_str().to_string(),
            STAGE_STATS_NEUTRAL.as_str().to_string(),
        ]
    } else {
        required_id_catalog()
    };
    if options.paired && options.enable_correct {
        stages.push(STAGE_CORRECT.as_str().to_string());
    }
    if options.mode == FastqPipelineMode::Shotgun && options.paired && options.enable_merge {
        stages.push(STAGE_MERGE.as_str().to_string());
    }
    if options.mode == FastqPipelineMode::Amplicon
        && !stages
            .iter()
            .any(|stage| stage == STAGE_ASV_INFERENCE.as_str())
    {
        stages.push(STAGE_OTU_CLUSTERING.as_str().to_string());
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
    pub pipeline_stages: Vec<StageId>,
    pub pipeline_tools: Vec<bijux_dna_core::ids::ToolId>,
    pub stage_skips: Vec<serde_json::Value>,
}

#[must_use]
pub fn apply_preprocess_policy(
    pipeline_stages: Vec<StageId>,
    pipeline_tools: Vec<bijux_dna_core::ids::ToolId>,
) -> PreprocessPolicyDecision {
    let mut pipeline_stages = pipeline_stages;
    let mut pipeline_tools = pipeline_tools;
    let mut stage_skips = Vec::new();

    if let (Some(trim_idx), Some(filter_idx)) = (
        pipeline_stages
            .iter()
            .position(|stage| stage == &STAGE_TRIM),
        pipeline_stages
            .iter()
            .position(|stage| stage == &STAGE_FILTER),
    ) {
        let trim_tool = pipeline_tools.get(trim_idx).map(|tool| tool.as_str());
        let filter_tool = pipeline_tools.get(filter_idx).map(|tool| tool.as_str());
        if trim_tool == Some(id_catalog::TOOL_FASTP) && filter_tool == Some(id_catalog::TOOL_FASTP)
        {
            let skipped_stage = pipeline_stages.remove(filter_idx);
            let skipped_tool = pipeline_tools.remove(filter_idx);
            stage_skips.push(serde_json::json!({
                "stage_id": skipped_stage.as_str(),
                "tool_id": skipped_tool.as_str(),
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
        let thresholds = bijux_dna_domain_fastq::thresholds_from_env();
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
) -> crate::tool_adapters::stages::pre::preprocess::PreprocessPlan {
    crate::tool_adapters::stages::pre::preprocess::PreprocessPlan {
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        stages: pipeline.stages,
        enable_contaminant_removal: args.enable_contaminant_removal,
        pipeline_mode: match args.mode {
            crate::selection::args::FastqPlannerMode::Shotgun => FastqPipelineMode::Shotgun,
            crate::selection::args::FastqPlannerMode::EdnaAmplicon
            | crate::selection::args::FastqPlannerMode::PollenAmplicon => {
                FastqPipelineMode::Amplicon
            }
        },
    }
}

#[must_use]
pub fn resolve_preprocess_pipeline(
    args: &crate::selection::args::BenchFastqPreprocessArgs,
    decisions: &PreprocessDecisions,
) -> PipelineSpec {
    let amplicon_only = [
        "fastq.primer_normalization",
        "fastq.chimera_detection",
        "fastq.asv_inference",
        "fastq.otu_clustering",
        "fastq.abundance_normalization",
    ];
    let shotgun_mode = args.mode == crate::selection::args::FastqPlannerMode::Shotgun;
    let enable_merge = decisions.enable_merge;
    let enable_correct = decisions.enable_correct;
    let enable_qc_post = !args.no_qc_post;
    let enable_screen = args.contaminant_preset.is_some();
    if let Some(profile_id) = args.profile.as_deref() {
        match bijux_dna_pipelines::registry::profile_by_id(
            bijux_dna_pipelines::Domain::Fastq,
            profile_id,
        ) {
            Ok(profile) => {
                let mut stages: Vec<String> = fastq_pipeline_id_catalog(profile.id.as_str());
                stages = apply_layout_branching(stages, args.r2.is_some());
                if !shotgun_mode {
                    stages.retain(|stage| stage != "fastq.polyg_tailing");
                }
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
                if shotgun_mode {
                    stages.retain(|stage| !amplicon_only.contains(&stage.as_str()));
                }
                PipelineSpec { stages }
            }
            Err(err) => {
                eprintln!("unknown fastq profile {profile_id}: {err}; using default pipeline");
                let mut spec = default_pipeline_spec(DefaultPipelineOptions {
                    paired: args.r2.is_some(),
                    enable_merge,
                    enable_correct,
                    enable_qc_post,
                    enable_screen,
                    mode: if args.mode == crate::selection::args::FastqPlannerMode::Shotgun {
                        FastqPipelineMode::Shotgun
                    } else {
                        FastqPipelineMode::Amplicon
                    },
                });
                spec.stages = apply_layout_branching(spec.stages, args.r2.is_some());
                if !shotgun_mode {
                    spec.stages.retain(|stage| stage != "fastq.polyg_tailing");
                }
                if shotgun_mode {
                    spec.stages
                        .retain(|stage| !amplicon_only.contains(&stage.as_str()));
                }
                spec
            }
        }
    } else {
        let mut spec = default_pipeline_spec(DefaultPipelineOptions {
            paired: args.r2.is_some(),
            enable_merge,
            enable_correct,
            enable_qc_post,
            enable_screen,
            mode: if args.mode == crate::selection::args::FastqPlannerMode::Shotgun {
                FastqPipelineMode::Shotgun
            } else {
                FastqPipelineMode::Amplicon
            },
        });
        spec.stages = apply_layout_branching(spec.stages, args.r2.is_some());
        if !shotgun_mode {
            spec.stages.retain(|stage| stage != "fastq.polyg_tailing");
        }
        if shotgun_mode {
            spec.stages
                .retain(|stage| !amplicon_only.contains(&stage.as_str()));
        }
        spec
    }
}

include!("planner_fastq_pipeline_decisions.rs");
