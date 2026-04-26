use anyhow::{anyhow, ensure, Result};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_dna_stage_contract::{StagePlugin, StagePluginOutputV1};

mod invocation;
mod output;

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
        validate_command_template(plan)?;
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

fn validate_command_template(plan: &StagePlanV1) -> Result<()> {
    ensure!(
        !plan.command.template.is_empty(),
        "BAM stage {} has empty command template",
        plan.stage_id.as_str()
    );
    ensure!(
        plan.command.template.iter().all(|arg| !arg.trim().is_empty()),
        "BAM stage {} has blank command template argument",
        plan.stage_id.as_str()
    );
    Ok(())
}
