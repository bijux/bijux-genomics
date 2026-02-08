use anyhow::Result;
use bijux_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_stage_contract::{StageInvocationV1, StagePlugin, StagePluginOutputV1};

use crate::metrics;

#[allow(dead_code)]
pub struct FastqStagePlugin;

impl StagePlugin for FastqStagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool {
        stage_id.starts_with("fastq.")
    }

    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1> {
        Ok(StageInvocationV1 {
            command: plan.command.template.clone(),
            env: std::collections::BTreeMap::new(),
            expected_outputs: plan.io.outputs.clone(),
        })
    }

    fn parse_outputs(
        &self,
        plan: &StagePlanV1,
        _outputs: &[ArtifactRef],
    ) -> Result<StagePluginOutputV1> {
        let input_paths: Vec<std::path::PathBuf> = plan
            .io
            .inputs
            .iter()
            .map(|input| input.path.clone())
            .collect();
        let output_paths: Vec<std::path::PathBuf> = plan
            .io
            .outputs
            .iter()
            .map(|output| output.path.clone())
            .collect();
        let envelope = metrics::build_metrics_envelope(plan, &input_paths, &output_paths)?;
        Ok(StagePluginOutputV1 {
            metrics: envelope,
            artifacts: Vec::new(),
            report_parts: Vec::new(),
            warnings: Vec::new(),
            invariants: Vec::new(),
            verdict: None,
            event_hints: Vec::new(),
        })
    }
}
