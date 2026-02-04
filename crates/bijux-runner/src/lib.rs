//! Owner: bijux-runner
//! Runner abstraction with docker/local backends.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use bijux_core::StagePlanV1;

pub mod docker;
pub mod local;

#[derive(Debug, Clone)]
pub struct Invocation {
    pub stage: StagePlanV1,
    pub attempt: u32,
}

#[derive(Debug, Clone)]
pub struct Artifact {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone)]
pub struct RunnerResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub artifacts: Vec<Artifact>,
}

pub trait Runner {
    /// # Errors
    /// Returns an error if the runner cannot execute the invocation or capture results.
    fn run(&self, invocation: &Invocation) -> Result<RunnerResult>;
}

pub mod primitives {
    pub use crate::docker::executor::{
        docker_logs, docker_rm, docker_stats_mb, docker_wait, docker_wait_timeout, execute_plan,
        execute_plan_with_timeout, parse_mem_to_mb, resolve_image_for_run, ExecutionAssessment,
        StageExecutionPlan,
    };
    pub use crate::docker::replay::replay_run;
    pub use crate::docker::support::build_tool_execution_spec;
}
