use anyhow::Result;
use bijux_dna_core::contract::{ExecutionGraph, RunRecordV1};
use bijux_dna_runtime::run_layout::RunLayout;
use bijux_dna_runtime::Runner;

use crate::{executor, CancellationToken, EngineConfig, EngineHooks};

pub struct Engine {
    config: EngineConfig,
}

impl Engine {
    #[must_use]
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }
}

impl Engine {
    /// # Errors
    /// Returns an error if validation or execution fails.
    pub fn execute(
        &self,
        graph: &ExecutionGraph,
        runner: &dyn Runner,
        _layout: &RunLayout,
        hooks: Option<&dyn EngineHooks>,
        cancel: Option<&CancellationToken>,
    ) -> Result<RunRecordV1> {
        let mut graph = graph.clone();
        if let Some(timeout) = self.config.step_timeout_s {
            graph = graph.with_step_timeout(Some(timeout));
        }
        if self.config.deterministic_scheduler {
            graph = graph.with_deterministic_scheduler(true);
        }
        if let Some(policy) = self.config.retry_policy.clone() {
            graph = graph.with_retry_policy(policy);
        }
        executor::execute_plan(&graph, runner, hooks, cancel)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}
