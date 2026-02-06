use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use bijux_core::plan::execution_graph::ExecutionStep;

#[derive(Debug, Clone)]
pub struct Invocation {
    pub step: ExecutionStep,
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
