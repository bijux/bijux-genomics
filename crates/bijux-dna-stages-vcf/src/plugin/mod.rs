use anyhow::{anyhow, ensure, Result};
use bijux_dna_stage_contract::{
    ArtifactRef, StageInvocationV1, StagePlanV1, StagePlugin, StagePluginOutputV1,
};

mod invocation;
mod output;

pub struct VcfStagePlugin;

impl StagePlugin for VcfStagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool {
        invocation::handles_vcf_stage(stage_id)
    }

    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1> {
        if !self.handles_stage(plan.stage_id.as_str()) {
            return Err(anyhow!("unsupported VCF stage {}", plan.stage_id.as_str()));
        }
        ensure!(
            !plan.command.template.is_empty(),
            "VCF stage {} has empty command template",
            plan.stage_id.as_str()
        );
        ensure!(
            plan.command.template.iter().all(|argument| !argument.trim().is_empty()),
            "VCF stage {} has blank command template argument",
            plan.stage_id.as_str()
        );
        Ok(invocation::materialize_stage_invocation(plan))
    }

    fn parse_outputs(
        &self,
        plan: &StagePlanV1,
        outputs: &[ArtifactRef],
    ) -> Result<StagePluginOutputV1> {
        if !self.handles_stage(plan.stage_id.as_str()) {
            return Err(anyhow!("unsupported VCF stage {}", plan.stage_id.as_str()));
        }
        output::parse_stage_outputs(plan, outputs)
    }
}
