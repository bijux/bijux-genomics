//! Owner: bijux-runner
//! Runner abstraction with docker/local backends.

pub use bijux_runtime::runner::{Artifact, Invocation, Runner, RunnerResult};

pub mod docker;
pub mod execute;
pub mod local;

pub mod primitives {
    pub use crate::docker::executor::{
        docker_logs, docker_rm, docker_stats_mb, docker_wait, docker_wait_timeout, execute_plan,
        execute_plan_with_timeout, parse_mem_to_mb, resolve_image_for_run, ExecutionAssessment,
        StageExecutionPlan,
    };
    pub use crate::docker::replay::replay_run;
    pub use crate::docker::support::build_tool_execution_spec;
    pub use crate::execute::{execute_stage_plan, StageResultV1};
}
