use anyhow::Result;
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
        invocation::materialize_stage_invocation(plan)
    }

    fn parse_outputs(
        &self,
        plan: &StagePlanV1,
        outputs: &[ArtifactRef],
    ) -> Result<StagePluginOutputV1> {
        output::parse_stage_outputs(plan, outputs)
    }
}
