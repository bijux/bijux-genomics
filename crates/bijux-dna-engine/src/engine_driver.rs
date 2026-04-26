use anyhow::{ensure, Result};
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
        layout: &RunLayout,
        hooks: Option<&dyn EngineHooks>,
        cancel: Option<&CancellationToken>,
    ) -> Result<RunRecordV1> {
        self.config.validate()?;
        validate_run_layout(layout)?;
        let graph = apply_engine_config(graph, &self.config);
        executor::execute_plan(&graph, runner, hooks, cancel)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

fn validate_run_layout(layout: &RunLayout) -> Result<()> {
    ensure!(layout.run_dir.is_dir(), "run layout run_dir must exist: {}", layout.run_dir.display());
    ensure!(
        layout.stages_dir.is_dir(),
        "run layout stages_dir must exist: {}",
        layout.stages_dir.display()
    );
    ensure!(
        layout.summary_dir.is_dir(),
        "run layout summary_dir must exist: {}",
        layout.summary_dir.display()
    );

    for path in [
        &layout.assessment_path,
        &layout.manifest_path,
        &layout.environment_path,
        &layout.metadata_path,
        &layout.events_path,
    ] {
        if let Some(parent) = path.parent() {
            ensure!(parent.is_dir(), "run layout file parent must exist for {}", path.display());
        }
    }

    Ok(())
}
