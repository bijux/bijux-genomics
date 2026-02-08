//! Owner: bijux-runner
//! Runner abstraction with docker/local backends.

pub use bijux_runtime::{Artifact, Invocation, Runner, RunnerResult};
use std::time::Duration;

pub mod backend;
pub mod execute;
pub mod runner_core;

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
        let result = execute::execute_step(
            &invocation.step,
            bijux_environment::api::RuntimeKind::Docker,
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

mod primitives {
    pub use crate::backend::docker::execution_spec::build_tool_execution_spec;
    pub use crate::backend::docker::executor::{resolve_image_for_run, ExecutionAssessment};
    pub use crate::backend::docker::replay::replay_run;
    pub use crate::execute::{execute_observer_command, execute_step, StageResultV1};
}

pub use primitives::{
    build_tool_execution_spec, execute_observer_command, execute_step, replay_run,
    resolve_image_for_run, ExecutionAssessment, StageResultV1,
};
