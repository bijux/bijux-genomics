use bijux_core::domain::PipelineSpec;
use bijux_core::{
    ArtifactRef, ContainerImageRefV1, StageIO, StageId, StagePlanV1, StageVersion,
    ToolExecutionSpecV1,
};
use bijux_domain_fastq::assess_merge_suitability;
use bijux_domain_fastq::params::{preprocess::PreprocessEffectiveParams, PairedMode};

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
    let pipeline = crate::fastq_default_pipeline(crate::DefaultPipelineOptions {
        paired: args.r2.is_some(),
        enable_merge,
        enable_correct,
        enable_qc_post: !args.no_qc_post,
        enable_screen: args.contaminant_preset.is_some(),
    });
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

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn plan_preprocess_pipeline<F>(
    stages: &[String],
    tools: &[ToolExecutionSpecV1],
    aux_images: &std::collections::BTreeMap<String, ContainerImageRefV1>,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    enable_contaminant_removal: bool,
    r1: &std::path::Path,
    r2: Option<&std::path::Path>,
    mut out_dir_for_stage: F,
) -> anyhow::Result<Vec<StagePlanV1>>
where
    F: FnMut(
        &str,
        &ToolExecutionSpecV1,
        &std::path::Path,
        Option<&std::path::Path>,
    ) -> anyhow::Result<std::path::PathBuf>,
{
    if stages.len() != tools.len() {
        return Err(anyhow::anyhow!(
            "pipeline stages/tools length mismatch: {} vs {}",
            stages.len(),
            tools.len()
        ));
    }
    let mut current_r1 = r1.to_path_buf();
    let raw_r1 = r1.to_path_buf();
    let mut current_r2 = r2.map(|path| path.to_path_buf());
    let mut plans = Vec::new();
    for (stage, tool) in stages.iter().zip(tools.iter()) {
        let out_dir = out_dir_for_stage(stage, tool, &current_r1, current_r2.as_deref())?;
        let (plan, next_r1, next_r2, stage_version) = match stage.as_str() {
            "fastq.detect_adapters" => {
                let plan = crate::fastq::detect_adapters::plan(tool, &current_r1, &out_dir);
                (
                    plan.clone(),
                    current_r1.clone(),
                    current_r2.clone(),
                    crate::fastq::detect_adapters::STAGE_VERSION,
                )
            }
            "fastq.trim" => {
                let plan = crate::fastq::trim::plan(
                    tool,
                    &current_r1,
                    &out_dir,
                    adapter_bank,
                    polyx_bank,
                    contaminant_bank,
                )?;
                (
                    plan.clone(),
                    plan.io.outputs[0].path.clone(),
                    None,
                    crate::fastq::trim::STAGE_VERSION,
                )
            }
            "fastq.filter" => {
                let mut filter_options = crate::fastq::filter::FilterPlanOptions::default();
                if adapter_bank.is_some() {
                    filter_options.redundant_filters.push("adapter".to_string());
                }
                if polyx_bank.is_some() {
                    filter_options.redundant_filters.push("polyx".to_string());
                }
                if enable_contaminant_removal && contaminant_bank.is_some() {
                    filter_options.kmer_ref = crate::fastq::filter::default_kmer_ref();
                }
                let plan = crate::fastq::filter::plan_filter(
                    tool,
                    &current_r1,
                    &out_dir,
                    &filter_options,
                )?;
                (
                    plan.clone(),
                    plan.io.outputs[0].path.clone(),
                    None,
                    crate::fastq::filter::STAGE_VERSION,
                )
            }
            "fastq.validate_pre" => {
                let plan = crate::fastq::validate_pre::plan(tool, &current_r1, &out_dir);
                (
                    plan.clone(),
                    current_r1.clone(),
                    current_r2.clone(),
                    crate::fastq::validate_pre::STAGE_VERSION,
                )
            }
            "fastq.merge" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("merge requires r2"))?;
                let plan = crate::fastq::merge::plan_merge(tool, &current_r1, r2, &out_dir)?;
                (
                    plan.clone(),
                    plan.io.outputs[0].path.clone(),
                    None,
                    crate::fastq::merge::STAGE_VERSION,
                )
            }
            "fastq.correct" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("correct requires r2"))?;
                let plan = crate::fastq::correct::plan_correct(tool, &current_r1, r2, &out_dir)?;
                (
                    plan.clone(),
                    plan.io.outputs[0].path.clone(),
                    Some(plan.io.outputs[1].path.clone()),
                    crate::fastq::correct::STAGE_VERSION,
                )
            }
            "fastq.umi" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("umi requires r2"))?;
                let plan = crate::fastq::umi::plan_umi(tool, &current_r1, r2, &out_dir)?;
                (
                    plan.clone(),
                    plan.io.outputs[0].path.clone(),
                    Some(plan.io.outputs[1].path.clone()),
                    crate::fastq::umi::STAGE_VERSION,
                )
            }
            "fastq.qc_post" => {
                let mut stage_aux_images = std::collections::BTreeMap::new();
                if tool.tool_id.0 == "multiqc" {
                    for aux_tool in crate::fastq::qc_post::aux_tool_ids() {
                        if let Some(image) = aux_images.get(*aux_tool) {
                            stage_aux_images.insert(aux_tool.to_string(), image.clone());
                        }
                    }
                }
                let plan = crate::fastq::qc_post::plan_qc_post(
                    tool,
                    &current_r1,
                    &out_dir,
                    stage_aux_images,
                    Some(raw_r1.as_path()),
                )?;
                (
                    plan.clone(),
                    current_r1.clone(),
                    current_r2.clone(),
                    crate::fastq::qc_post::STAGE_VERSION,
                )
            }
            "fastq.screen" => {
                let plan = crate::fastq::screen::plan_screen(tool, &current_r1, &out_dir)?;
                (
                    plan.clone(),
                    current_r1.clone(),
                    current_r2.clone(),
                    crate::fastq::screen::STAGE_VERSION,
                )
            }
            "fastq.stats_neutral" => {
                let plan =
                    crate::fastq::stats_neutral::plan_stats_neutral(tool, &current_r1, &out_dir)?;
                (
                    plan.clone(),
                    current_r1.clone(),
                    current_r2.clone(),
                    crate::fastq::stats_neutral::STAGE_VERSION,
                )
            }
            _ => return Err(anyhow::anyhow!("unsupported stage {stage}")),
        };
        let mut exec_plan = plan;
        exec_plan.stage_id = StageId(stage.clone());
        exec_plan.stage_version = stage_version;
        exec_plan.out_dir = out_dir;
        plans.push(exec_plan);
        current_r1 = next_r1;
        current_r2 = next_r2;
    }
    Ok(plans)
}

#[must_use]
pub fn plan_preprocess_stage(plan: &PreprocessPlan, tool: &ToolExecutionSpecV1) -> StagePlanV1 {
    let mut inputs = vec![ArtifactRef {
        name: "reads_r1".to_string(),
        path: plan.r1.clone(),
    }];
    if let Some(r2) = &plan.r2 {
        inputs.push(ArtifactRef {
            name: "reads_r2".to_string(),
            path: r2.clone(),
        });
    }
    let effective_params = PreprocessEffectiveParams {
        paired_mode: PairedMode::from_has_r2(plan.r2.is_some()),
        threads: tool.resources.threads,
        stages: plan.pipeline.stages.clone(),
        enable_contaminant_removal: plan.enable_contaminant_removal,
    };
    StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs,
            outputs: Vec::new(),
        },
        out_dir: plan
            .r1
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf(),
        params: serde_json::json!({
            "r1": plan.r1,
            "r2": plan.r2,
            "stages": plan.pipeline.stages,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .expect("serialize preprocess effective params"),
        aux_images: std::collections::BTreeMap::new(),
    }
}
