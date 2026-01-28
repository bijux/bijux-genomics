mod docker;
mod image;
mod run_merge;
mod run_tool;
mod run_validate;

pub use docker::{
    docker_logs, docker_rm, docker_stats_mb, docker_wait, docker_wait_timeout, parse_mem_to_mb,
};
pub use image::resolve_image_for_run;
pub use run_merge::{run_merge_container, run_merge_container_with_timeout, MergeExecutionOutput};
pub use run_tool::{run_tool_container, run_tool_container_with_timeout, ExecutionOutput};
pub use run_validate::{
    run_multiqc_container, run_multiqc_container_with_timeout, run_validate_container,
    run_validate_container_with_timeout,
};

use anyhow::Result;

use crate::types::{RunPlan, StageResult};

pub fn execute_plan(plan: &RunPlan) -> Result<StageResult> {
    if crate::types::trace_enabled() {
        println!(
            "[engine][executor] stage={} tool={} runner={}",
            plan.invocation.stage_id, plan.invocation.tool_id, plan.runner
        );
    }
    Ok(StageResult {
        invocation: plan.invocation.clone(),
        exit_code: 0,
        stdout: String::new(),
        stderr: String::new(),
        outputs: Vec::new(),
    })
}
