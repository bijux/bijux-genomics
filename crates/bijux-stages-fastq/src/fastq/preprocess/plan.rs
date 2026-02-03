use bijux_core::domain::PipelineSpec;
use bijux_core::StageVersion;
use bijux_domain_fastq::assess_merge_suitability;
use bijux_pipelines::registry;
use bijux_pipelines::Domain;

pub const STAGE_ID: &str = "fastq.preprocess";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone)]
pub struct PreprocessPlan {
    pub r1: std::path::PathBuf,
    pub r2: Option<std::path::PathBuf>,
    pub pipeline: PipelineSpec,
    pub merge_decision: Option<MergeDecisionTrace>,
    pub correct_decision: Option<CorrectDecisionTrace>,
    pub enable_contaminant_removal: bool,
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
pub fn plan_preprocess(args: &crate::args::BenchFastqPreprocessArgs) -> PreprocessPlan {
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
    let pipeline = if let Some(profile_id) = args.profile.as_deref() {
        match registry::profile_by_id(Domain::Fastq, profile_id) {
            Ok(profile) => {
                let mut stages: Vec<String> =
                    profile.graph.into_iter().map(|node| node.stage_id).collect();
                if !enable_merge {
                    stages.retain(|stage| stage != "fastq.merge");
                }
                if !enable_correct {
                    stages.retain(|stage| stage != "fastq.correct");
                }
                if args.no_qc_post {
                    stages.retain(|stage| stage != "fastq.qc_post");
                }
                if args.contaminant_preset.is_none() {
                    stages.retain(|stage| stage != "fastq.screen");
                }
                bijux_core::domain::PipelineSpec { stages }
            }
            Err(err) => {
                eprintln!("unknown fastq profile {profile_id}: {err}; using default pipeline");
                bijux_pipelines::fastq::fastq_default_pipeline_spec(
                    bijux_pipelines::fastq::DefaultPipelineOptions {
                        paired: args.r2.is_some(),
                        enable_merge,
                        enable_correct,
                        enable_qc_post: !args.no_qc_post,
                        enable_screen: args.contaminant_preset.is_some(),
                    },
                )
            }
        }
    } else {
        bijux_pipelines::fastq::fastq_default_pipeline_spec(
            bijux_pipelines::fastq::DefaultPipelineOptions {
                paired: args.r2.is_some(),
                enable_merge,
                enable_correct,
                enable_qc_post: !args.no_qc_post,
                enable_screen: args.contaminant_preset.is_some(),
            },
        )
    };
    PreprocessPlan {
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        pipeline,
        merge_decision,
        correct_decision,
        enable_contaminant_removal: args.enable_contaminant_removal,
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
