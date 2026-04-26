use anyhow::Result;
use bijux_dna_core::contract::{ExecutionGraph, RunRecordV1};
use bijux_dna_runtime::run_layout::RunLayout;
use bijux_dna_runtime::Runner;

use crate::engine_config::apply_engine_config;
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
        self.config.validate()?;
        let graph = apply_engine_config(graph, &self.config);
        executor::execute_plan(&graph, runner, hooks, cancel)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}
