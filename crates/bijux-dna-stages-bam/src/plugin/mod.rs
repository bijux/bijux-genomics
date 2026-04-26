use anyhow::{anyhow, Result};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_dna_stage_contract::{StagePlugin, StagePluginOutputV1};

mod invocation;
mod output;

#[allow(dead_code)]
pub struct BamStagePlugin;

impl StagePlugin for BamStagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool {
        invocation::handles_bam_stage(stage_id)
    }

    fn materialize(
        &self,
        plan: &StagePlanV1,
    ) -> Result<bijux_dna_stage_contract::StageInvocationV1> {
        if !self.handles_stage(plan.stage_id.as_str()) {
            return Err(anyhow!("unsupported BAM stage {}", plan.stage_id.as_str()));
        }
        if plan.command.template.is_empty() {
            return Err(anyhow!("BAM stage {} has empty command template", plan.stage_id.as_str()));
        }
        Ok(invocation::materialize_stage_invocation(plan))
    }

    fn parse_outputs(
        &self,
        plan: &StagePlanV1,
        outputs: &[ArtifactRef],
    ) -> Result<StagePluginOutputV1> {
        if !self.handles_stage(plan.stage_id.as_str()) {
            return Err(anyhow!("unsupported BAM stage {}", plan.stage_id.as_str()));
        }
        output::parse_stage_outputs(plan, outputs)
    }
}
