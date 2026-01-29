use bijux_core::domain::PipelineSpec;
use bijux_core::{StageId, StageVersion};

use crate::plan::{ArtifactRef, StageIO, StagePlan};

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
