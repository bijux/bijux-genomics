pub mod execution_plan;
pub mod run;
pub mod stage_plan;
pub mod stage_plugin;

#[allow(unused_imports)]
pub use execution_plan::*;
#[allow(unused_imports)]
pub use run::*;
#[allow(unused_imports)]
pub use stage_plan::*;
#[allow(unused_imports)]
pub use stage_plugin::*;

use bijux_core::execution::execution_graph::ExecutionStep;

#[must_use]
pub fn execution_step_from_stage_plan(plan: &crate::StagePlanV1) -> ExecutionStep {
    ExecutionStep {
        step_id: plan.stage_id.clone(),
        command: plan.command.clone(),
        image: plan.image.clone(),
        resources: plan.resources.clone(),
        io: plan.io.clone(),
        out_dir: plan.out_dir.clone(),
        aux_images: plan.aux_images.clone(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    }
}

pub use bijux_core::contract::{ArtifactRef, StageIO};

#[allow(unused_imports)]
pub use execution_plan::*;
#[allow(unused_imports)]
pub use run::*;
#[allow(unused_imports)]
pub use stage_plan::*;
#[allow(unused_imports)]
pub use stage_plugin::*;
