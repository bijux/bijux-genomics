use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_core::ids::StepId;

use super::contract::StagePlanV1;

#[must_use]
pub fn execution_step_from_stage_plan(plan: &StagePlanV1) -> ExecutionStep {
    execution_step_from_stage_plan_with_step_id(
        plan,
        plan.stage_instance_id
            .clone()
            .unwrap_or_else(|| StepId::new(plan.stage_id.to_string())),
    )
}

#[must_use]
pub fn execution_step_from_stage_plan_with_step_id(
    plan: &StagePlanV1,
    step_id: StepId,
) -> ExecutionStep {
    let expected_artifact_ids = plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.clone())
        .collect();
    let metrics_schema_ids =
        bijux_dna_core::metrics::metrics_schema_for_stage(plan.stage_id.as_str())
            .map(|schema| vec![schema.schema.to_string()])
            .unwrap_or_default();
    ExecutionStep {
        step_id,
        stage_id: plan.stage_id.clone(),
        command: plan.command.clone(),
        image: plan.image.clone(),
        resources: plan.resources.clone(),
        io: plan.io.clone(),
        out_dir: plan.out_dir.clone(),
        aux_images: plan.aux_images.clone(),
        expected_artifact_ids,
        metrics_schema_ids,
    }
}
