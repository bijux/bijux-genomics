#![allow(clippy::too_many_arguments)]

use bijux_dna_core::contract::PipelineSpec;
use bijux_dna_core::prelude::input_assessment::{assess_input_dir, FastqLayout};
use bijux_dna_core::prelude::StageId;
use bijux_dna_domain_fastq::{
    assess_merge_suitability, preprocess_pipeline_graph_for_stage_order, STAGE_CORRECT_ERRORS,
    STAGE_MERGE_PAIRS, STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY, STAGE_TRIM_TERMINAL_DAMAGE,
};

use crate::planner::{apply_layout_branching, estimate_mean_q};
use crate::selection::args::BenchFastqPreprocessArgs;
use crate::tool_adapters::stages::pre::preprocess::PreprocessPlan;
use crate::{default_pipeline_spec, pipeline_spec_from_stage_catalog, DefaultPipelineOptions};

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
pub fn preprocess_decisions(args: &BenchFastqPreprocessArgs) -> PreprocessDecisions {
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
    if enable_correct {
        correct_decision = Some(CorrectDecisionTrace {
            enabled: true,
            auto_enabled: false,
            reason: "error correction enabled by user flag".to_string(),
            mean_q_estimate: None,
        });
    } else {
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
    }
    PreprocessDecisions { enable_merge, enable_correct, merge_decision, correct_decision }
}

#[must_use]
pub fn plan_preprocess(args: &BenchFastqPreprocessArgs, pipeline: &PipelineSpec) -> PreprocessPlan {
    PreprocessPlan {
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        stages: pipeline.stage_catalog(),
        enable_contaminant_removal: args.enable_contaminant_removal,
        pipeline_mode: args.mode.pipeline_mode(),
    }
}

#[derive(Debug, Clone, Copy)]
struct PipelineFilterOptions {
    paired: bool,
    shotgun_mode: bool,
    enable_merge: bool,
    enable_correct: bool,
    enable_qc_post: bool,
    enable_screen: bool,
    enable_terminal_damage_trim: bool,
}

#[must_use]
pub fn resolve_preprocess_pipeline(
    args: &BenchFastqPreprocessArgs,
    decisions: &PreprocessDecisions,
) -> PipelineSpec {
    let amplicon_only = [
        "fastq.normalize_primers",
        "fastq.remove_chimeras",
        "fastq.infer_asvs",
        "fastq.cluster_otus",
        "fastq.normalize_abundance",
    ];
    let shotgun_mode = args.mode.is_shotgun_family();
    let enable_merge = decisions.enable_merge;
    let enable_correct = decisions.enable_correct;
    let enable_qc_post = !args.no_qc_post;
    let enable_screen = args.contaminant_preset.is_some();
    let enable_terminal_damage_trim = args.mode.admits_terminal_damage_trim();
    if let Some(profile_id) = args.profile.as_deref() {
        match bijux_dna_pipelines::registry::profile_by_id(
            bijux_dna_pipelines::Domain::Fastq,
            profile_id,
        ) {
            Ok(profile) => filter_preprocess_pipeline(
                &pipeline_spec_from_stage_catalog(
                    profile
                        .capabilities
                        .required_stages
                        .iter()
                        .filter(|stage| stage.starts_with(bijux_dna_domain_fastq::STAGE_PREFIX))
                        .map(|stage| (*stage).to_string())
                        .collect(),
                    args.mode.pipeline_mode(),
                ),
                PipelineFilterOptions {
                    paired: args.r2.is_some(),
                    shotgun_mode,
                    enable_merge,
                    enable_correct,
                    enable_qc_post,
                    enable_screen: enable_screen
                        || profile
                            .capabilities
                            .required_stages
                            .iter()
                            .any(|stage| stage == STAGE_SCREEN_TAXONOMY.as_str()),
                    enable_terminal_damage_trim,
                },
                &amplicon_only,
            ),
            Err(err) => {
                eprintln!("unknown fastq profile {profile_id}: {err}; using default pipeline");
                filter_preprocess_pipeline(
                    &default_pipeline_spec(DefaultPipelineOptions {
                        paired: args.r2.is_some(),
                        enable_merge,
                        enable_correct,
                        enable_qc_post,
                        enable_screen,
                        mode: args.mode.pipeline_mode(),
                    }),
                    PipelineFilterOptions {
                        paired: args.r2.is_some(),
                        shotgun_mode,
                        enable_merge,
                        enable_correct,
                        enable_qc_post,
                        enable_screen,
                        enable_terminal_damage_trim,
                    },
                    &amplicon_only,
                )
            }
        }
    } else {
        filter_preprocess_pipeline(
            &default_pipeline_spec(DefaultPipelineOptions {
                paired: args.r2.is_some(),
                enable_merge,
                enable_correct,
                enable_qc_post,
                enable_screen,
                mode: args.mode.pipeline_mode(),
            }),
            PipelineFilterOptions {
                paired: args.r2.is_some(),
                shotgun_mode,
                enable_merge,
                enable_correct,
                enable_qc_post,
                enable_screen,
                enable_terminal_damage_trim,
            },
            &amplicon_only,
        )
    }
}

fn filter_preprocess_pipeline(
    spec: &PipelineSpec,
    options: PipelineFilterOptions,
    amplicon_only: &[&str],
) -> PipelineSpec {
    let mut allowed_stages = apply_layout_branching(spec.stage_catalog(), options.paired);
    if !options.shotgun_mode {
        allowed_stages.retain(|stage| stage != "fastq.trim_polyg_tails");
    }
    if !options.enable_merge {
        allowed_stages.retain(|stage| stage != STAGE_MERGE_PAIRS.as_str());
    }
    if !options.enable_correct {
        allowed_stages.retain(|stage| stage != STAGE_CORRECT_ERRORS.as_str());
    }
    if !options.enable_qc_post {
        allowed_stages.retain(|stage| stage != STAGE_REPORT_QC.as_str());
    }
    if !options.enable_screen {
        allowed_stages.retain(|stage| stage != STAGE_SCREEN_TAXONOMY.as_str());
    }
    if !options.enable_terminal_damage_trim {
        allowed_stages.retain(|stage| stage != STAGE_TRIM_TERMINAL_DAMAGE.as_str());
    }
    if options.shotgun_mode {
        allowed_stages.retain(|stage| !amplicon_only.contains(&stage.as_str()));
    }
    preprocess_pipeline_graph_for_stage_order(
        &allowed_stages.into_iter().map(StageId::new).collect::<Vec<_>>(),
    )
}
