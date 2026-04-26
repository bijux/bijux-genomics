use std::collections::BTreeMap;

use bijux_dna_domain_bam::BamStage;
use bijux_dna_stage_contract::{StageInvocationV1, StagePlanV1};

pub(super) fn handles_bam_stage(stage_id: &str) -> bool {
    BamStage::try_from(stage_id).is_ok()
}

pub(super) fn materialize_stage_invocation(plan: &StagePlanV1) -> StageInvocationV1 {
    StageInvocationV1 {
        command: plan.command.template.clone(),
        env: BTreeMap::new(),
        expected_outputs: plan.io.outputs.clone(),
    }
}
