use std::time::Duration;

use bijux_dna_runtime::{Artifact, Invocation, Runner, RunnerResult};

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
        let result = crate::step_runner::execute_step(
            &invocation.step,
            bijux_dna_environment::api::RuntimeKind::Docker,
            self.timeout,
        )?;
        let crate::step_runner::StageResultV1 {
            exit_code,
            runtime_s,
            outputs,
            metrics_path,
            stdout,
            stderr,
            ..
        } = result;
        let mut paths = outputs;
        if let Some(metrics_path) = metrics_path {
            paths.push(metrics_path);
        }
        let mut artifacts = Vec::new();
        for path in paths {
            if path.exists() {
                let sha256 = bijux_dna_infra::hash_file_sha256(&path)?;
                artifacts.push(Artifact { path, sha256 });
            }
        }
        Ok(RunnerResult {
            exit_code,
            stdout,
            stderr,
            duration: Duration::from_secs_f64(runtime_s),
            artifacts,
        })
    }
}
