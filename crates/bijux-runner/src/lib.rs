//! Owner: bijux-runner
//! Runner abstraction with docker/local backends.

pub use bijux_engine::runner::{Artifact, Invocation, Runner, RunnerResult};
use std::time::Duration;

pub mod docker;
pub mod execute;
pub mod local;

#[derive(Debug, Clone, Copy)]
pub struct DockerRunner {
    pub timeout: Option<Duration>,
}

impl DockerRunner {
    #[must_use]
    pub fn new(timeout: Option<Duration>) -> Self {
        Self { timeout }
    }
}

impl Runner for DockerRunner {
    fn run(&self, invocation: &Invocation) -> anyhow::Result<RunnerResult> {
        let result = execute::execute_stage_plan(
            &invocation.stage,
            bijux_environment::api::RunnerKind::Docker,
            self.timeout,
        )?;
        let mut paths = result.outputs.clone();
        if let Some(metrics_path) = result.metrics_path.clone() {
            paths.push(metrics_path);
        }
        let mut artifacts = Vec::new();
        for path in paths {
            if path.exists() {
                let sha256 = bijux_infra::hash_file_sha256(&path)?;
                artifacts.push(Artifact { path, sha256 });
            }
        }
        Ok(RunnerResult {
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
            duration: Duration::from_secs_f64(result.runtime_s),
            artifacts,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocalRunner;

impl Runner for LocalRunner {
    fn run(&self, _invocation: &Invocation) -> anyhow::Result<RunnerResult> {
        anyhow::bail!("local runner not implemented");
    }
}

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
