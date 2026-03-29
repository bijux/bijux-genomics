use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::contract::{
    ArtifactRef, ArtifactRole, ExecutionEdge, ExecutionGraph, ExecutionStep, StageIO,
    ToolConstraints,
};
use bijux_dna_core::contract::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec};
use bijux_dna_core::prelude::input_assessment::{assess_input_dir, FastqLayout};
use bijux_dna_core::prelude::{
    ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId, ToolExecutionSpecV1,
};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_domain_fastq::{
    assess_merge_suitability, canonical_amplicon_stage_order, canonical_stage_order,
    default_amplicon_preprocess_stage_order, default_shotgun_preprocess_stage_order,
    preprocess_pipeline_graph_for_stage_order,
};
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

pub const PLANNER_VERSION: &str = "bijux-dna-planner-fastq.v1";
pub const TOOL_SEQKIT: &str = "seqkit";
pub const STAGE_REPORT_AGGREGATE: StageId = StageId::from_static("report.aggregate");
pub const STAGE_COMPARE_STAGE_TOOLS: StageId =
    StageId::from_static("benchmark.compare_stage_tools");
pub const STAGE_SELECT_STAGE_TOOL: StageId = StageId::from_static("benchmark.select_stage_tool");
pub const STAGE_PREPROCESS_SUMMARY: StageId = StageId::from_static("fastq.preprocess");

pub use bijux_dna_domain_fastq::BenchResultsRepository;

mod plan_compose;
mod planner;
mod pipeline_defaults;
mod qc_contract;
mod report_stage;
mod selection;
pub mod tool_adapters;

pub use pipeline_defaults::{default_pipeline_spec, DefaultPipelineOptions};
pub use planner::*;
pub use report_stage::report_stage_step;
pub use selection::args;
use planner::{apply_layout_branching, estimate_mean_q};
pub(crate) use pipeline_defaults::{pipeline_spec_from_stage_catalog, required_id_catalog};

pub mod stage_api;

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
    let adapter_inference = pipeline_stages
        .iter()
        .zip(pipeline_tools.iter())
        .find(|(stage, _)| stage == &&STAGE_DETECT_ADAPTERS)
        .map(|(stage, tool)| {
            let trim_binding = pipeline_stages
                .iter()
                .zip(pipeline_tools.iter())
                .find(|(candidate_stage, _)| candidate_stage == &&STAGE_TRIM_READS)
                .map(|(candidate_stage, candidate_tool)| {
                    serde_json::json!({
                        "stage_id": candidate_stage.as_str(),
                        "tool_id": candidate_tool.as_str(),
                    })
                });
            serde_json::json!({
                "schema_version": "bijux.fastq.preprocess_policy.v1",
                "source_stage_id": stage.as_str(),
                "source_tool_id": tool.as_str(),
                "evidence_artifacts": ["adapter_report", "adapter_evidence_dir"],
                "handoff_mode": "runtime_evidence",
                "consumer_binding": trim_binding,
            })
        });
    PreprocessPolicyDecision {
        adapter_inference,
        adapter_bank_preset_override: None,
        pipeline_stages,
        pipeline_tools,
        stage_skips: Vec::new(),
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
    if !enable_correct {
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
        stages: pipeline.stage_catalog(),
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
        "fastq.normalize_primers",
        "fastq.remove_chimeras",
        "fastq.infer_asvs",
        "fastq.cluster_otus",
        "fastq.normalize_abundance",
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
            Ok(profile) => filter_preprocess_pipeline(
                pipeline_spec_from_stage_catalog(
                    fastq_pipeline_id_catalog(profile.id.as_str()),
                    if shotgun_mode {
                        FastqPipelineMode::Shotgun
                    } else {
                        FastqPipelineMode::Amplicon
                    },
                ),
                args.r2.is_some(),
                shotgun_mode,
                enable_merge,
                enable_correct,
                enable_qc_post,
                enable_screen,
                &amplicon_only,
            ),
            Err(err) => {
                eprintln!("unknown fastq profile {profile_id}: {err}; using default pipeline");
                filter_preprocess_pipeline(
                    default_pipeline_spec(DefaultPipelineOptions {
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
                    }),
                    args.r2.is_some(),
                    shotgun_mode,
                    enable_merge,
                    enable_correct,
                    enable_qc_post,
                    enable_screen,
                    &amplicon_only,
                )
            }
        }
    } else {
        filter_preprocess_pipeline(
            default_pipeline_spec(DefaultPipelineOptions {
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
            }),
            args.r2.is_some(),
            shotgun_mode,
            enable_merge,
            enable_correct,
            enable_qc_post,
            enable_screen,
            &amplicon_only,
        )
    }
}

fn filter_preprocess_pipeline(
    spec: PipelineSpec,
    paired: bool,
    shotgun_mode: bool,
    enable_merge: bool,
    enable_correct: bool,
    enable_qc_post: bool,
    enable_screen: bool,
    amplicon_only: &[&str],
) -> PipelineSpec {
    let mut allowed_stages = apply_layout_branching(spec.stage_catalog(), paired);
    if !shotgun_mode {
        allowed_stages.retain(|stage| stage != "fastq.trim_polyg_tails");
    }
    if !enable_merge {
        allowed_stages.retain(|stage| stage != STAGE_MERGE_PAIRS.as_str());
    }
    if !enable_correct {
        allowed_stages.retain(|stage| stage != STAGE_CORRECT_ERRORS.as_str());
    }
    if !enable_qc_post {
        allowed_stages.retain(|stage| stage != STAGE_REPORT_QC.as_str());
    }
    if !enable_screen {
        allowed_stages.retain(|stage| stage != STAGE_SCREEN_TAXONOMY.as_str());
    }
    if shotgun_mode {
        allowed_stages.retain(|stage| !amplicon_only.contains(&stage.as_str()));
    }
    preprocess_pipeline_graph_for_stage_order(
        &allowed_stages
            .into_iter()
            .map(StageId::new)
            .collect::<Vec<_>>(),
    )
}
