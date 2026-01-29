use bijux_core::domain::PipelineSpec;
use bijux_core::{StageId, StageVersion, ToolId};

use crate::plan::{ArtifactRef, StageIO, StagePlan, StagePlanJson};

pub const STAGE_ID: &str = "fastq.preprocess";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone)]
pub struct PreprocessPlan {
    pub r1: std::path::PathBuf,
    pub r2: Option<std::path::PathBuf>,
    pub pipeline: PipelineSpec,
}

#[must_use]
pub fn plan_preprocess(args: &crate::args::BenchFastqPreprocessArgs) -> PreprocessPlan {
    let pipeline = crate::fastq_default_pipeline(crate::DefaultPipelineOptions {
        paired: args.r2.is_some(),
        ..Default::default()
    });
    PreprocessPlan {
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        pipeline,
    }
}

#[derive(Debug, Clone)]
pub struct PreprocessStagePlanV1 {
    pub stage: StageId,
    pub tool: ToolId,
    pub plan: StagePlanJson,
    pub inputs: Vec<std::path::PathBuf>,
    pub outputs: Vec<std::path::PathBuf>,
}

#[allow(clippy::too_many_lines)]
pub fn plan_preprocess_pipeline<F>(
    stages: &[String],
    tools: &[String],
    r1: &std::path::Path,
    r2: Option<&std::path::Path>,
    mut out_dir_for_stage: F,
) -> anyhow::Result<Vec<PreprocessStagePlanV1>>
where
    F: FnMut(
        &str,
        &str,
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
    let mut current_r2 = r2.map(|path| path.to_path_buf());
    let mut plans = Vec::new();
    for (stage, tool) in stages.iter().zip(tools.iter()) {
        let out_dir = out_dir_for_stage(stage, tool, &current_r1, current_r2.as_deref())?;
        let (plan, inputs, outputs, next_r1, next_r2) = match stage.as_str() {
            "fastq.trim" => {
                let plan = crate::fastq::trim::plan(tool, &current_r1, &out_dir)?;
                let outputs = vec![plan.output.clone()];
                (
                    StagePlanJson::from_plan(&plan),
                    vec![current_r1.clone()],
                    outputs.clone(),
                    outputs[0].clone(),
                    None,
                )
            }
            "fastq.filter" => {
                let plan = crate::fastq::filter::plan_filter(tool, &current_r1, &out_dir)?;
                let outputs = vec![plan.output.clone()];
                (
                    StagePlanJson::from_plan(&plan),
                    vec![current_r1.clone()],
                    outputs.clone(),
                    outputs[0].clone(),
                    None,
                )
            }
            "fastq.validate_pre" => {
                let plan = crate::fastq::validate_pre::plan(tool, &current_r1, &out_dir);
                (
                    StagePlanJson::from_plan(&plan),
                    vec![current_r1.clone()],
                    Vec::new(),
                    current_r1.clone(),
                    current_r2.clone(),
                )
            }
            "fastq.merge" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("merge requires r2"))?;
                let plan = crate::fastq::merge::plan_merge(tool, &current_r1, r2, &out_dir)?;
                let outputs = vec![plan.output.clone()];
                (
                    StagePlanJson::from_plan(&plan),
                    vec![current_r1.clone(), r2.clone()],
                    outputs.clone(),
                    outputs[0].clone(),
                    None,
                )
            }
            "fastq.correct" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("correct requires r2"))?;
                let plan = crate::fastq::correct::plan_correct(tool, &current_r1, r2, &out_dir)?;
                let outputs = vec![plan.output_r1.clone(), plan.output_r2.clone()];
                (
                    StagePlanJson::from_plan(&plan),
                    vec![current_r1.clone(), r2.clone()],
                    outputs.clone(),
                    outputs[0].clone(),
                    Some(outputs[1].clone()),
                )
            }
            "fastq.umi" => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("umi requires r2"))?;
                let plan = crate::fastq::umi::plan_umi(tool, &current_r1, r2, &out_dir)?;
                let outputs = vec![plan.output_r1.clone(), plan.output_r2.clone()];
                (
                    StagePlanJson::from_plan(&plan),
                    vec![current_r1.clone(), r2.clone()],
                    outputs.clone(),
                    outputs[0].clone(),
                    Some(outputs[1].clone()),
                )
            }
            "fastq.qc_post" => {
                let plan = crate::fastq::qc_post::plan_qc_post(tool, &current_r1, &out_dir)?;
                (
                    StagePlanJson::from_plan(&plan),
                    vec![current_r1.clone()],
                    Vec::new(),
                    current_r1.clone(),
                    current_r2.clone(),
                )
            }
            "fastq.screen" => {
                let plan = crate::fastq::screen::plan_screen(tool, &current_r1, &out_dir)?;
                let outputs = vec![plan.report.clone()];
                (
                    StagePlanJson::from_plan(&plan),
                    vec![current_r1.clone()],
                    outputs.clone(),
                    current_r1.clone(),
                    current_r2.clone(),
                )
            }
            "fastq.stats_neutral" => {
                let plan = StagePlanJson {
                    stage_id: "fastq.stats_neutral".to_string(),
                    stage_version: "1".to_string(),
                    io: crate::plan::StageIO {
                        inputs: vec![crate::plan::ArtifactRef {
                            name: "reads_r1".to_string(),
                            path: current_r1.clone(),
                        }],
                        outputs: Vec::new(),
                    },
                    parameters: serde_json::json!({
                        "tool": tool,
                        "input": current_r1,
                        "out_dir": out_dir,
                    }),
                };
                (
                    plan,
                    vec![current_r1.clone()],
                    Vec::new(),
                    current_r1.clone(),
                    current_r2.clone(),
                )
            }
            _ => return Err(anyhow::anyhow!("unsupported stage {stage}")),
        };
        plans.push(PreprocessStagePlanV1 {
            stage: StageId(stage.clone()),
            tool: ToolId(tool.clone()),
            plan,
            inputs,
            outputs,
        });
        current_r1 = next_r1;
        current_r2 = next_r2;
    }
    Ok(plans)
}

impl StagePlan for PreprocessPlan {
    fn stage_id(&self) -> StageId {
        StageId(STAGE_ID.to_string())
    }

    fn stage_version(&self) -> StageVersion {
        STAGE_VERSION
    }

    fn outputs(&self) -> StageIO {
        let mut inputs = vec![ArtifactRef {
            name: "reads_r1".to_string(),
            path: self.r1.clone(),
        }];
        if let Some(r2) = &self.r2 {
            inputs.push(ArtifactRef {
                name: "reads_r2".to_string(),
                path: r2.clone(),
            });
        }
        StageIO {
            inputs,
            outputs: Vec::new(),
        }
    }

    fn parameters_json(&self) -> serde_json::Value {
        serde_json::json!({
            "r1": self.r1,
            "r2": self.r2,
            "stages": self.pipeline.stages,
        })
    }
}
