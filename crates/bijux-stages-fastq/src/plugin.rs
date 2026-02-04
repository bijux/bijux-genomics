use std::collections::BTreeMap;

use anyhow::Result;
use bijux_core::stage_plugin::{StageInvocationV1, StagePlugin, StagePluginOutputV1};
use bijux_core::{ArtifactRef, StagePlanV1};

pub struct FastqStagePlugin;

impl StagePlugin for FastqStagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool {
        stage_id.starts_with("fastq.")
    }

    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1> {
        Ok(StageInvocationV1 {
            command: plan.command.template.clone(),
            env: BTreeMap::new(),
            expected_outputs: plan.io.outputs.clone(),
        })
    }

    fn parse_outputs(&self, _plan: &StagePlanV1, _outputs: &[ArtifactRef]) -> Result<StagePluginOutputV1> {
        Ok(StagePluginOutputV1 {
            metrics: serde_json::json!({}),
            artifacts: Vec::new(),
        })
    }
}
