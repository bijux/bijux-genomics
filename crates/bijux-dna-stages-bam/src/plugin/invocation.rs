use std::collections::BTreeMap;

use anyhow::Result;
use bijux_dna_core::id_catalog;
use bijux_dna_stage_contract::{StageInvocationV1, StagePlanV1};

pub(super) fn handles_bam_stage(stage_id: &str) -> bool {
    stage_id.starts_with(id_catalog::BAM_PREFIX)
}

pub(super) fn materialize_stage_invocation(plan: &StagePlanV1) -> Result<StageInvocationV1> {
    Ok(StageInvocationV1 {
        command: plan.command.template.clone(),
        env: BTreeMap::new(),
        expected_outputs: plan.io.outputs.clone(),
    })
}
