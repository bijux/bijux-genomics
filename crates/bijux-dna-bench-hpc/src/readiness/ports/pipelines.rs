use crate::watchdog::{LocalDagWatchdogScenario, LocalDagWatchdogSimulationReport};
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalPipelineDagValidationReport;
use std::path::Path;
/// Supply typed local pipelines evidence; provider failures remain explicit readiness diagnostics.
pub trait PipelineEvidence {
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn validate_pipeline_dag_path(
        &self,
        repo_root: &Path,
        config_path: &Path,
        output_path: &Path,
    ) -> Result<BenchmarkLocalPipelineDagValidationReport>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn simulate_dag_watchdog_path(
        &self,
        repo_root: &Path,
        scenario: LocalDagWatchdogScenario,
        output_path: &Path,
    ) -> Result<LocalDagWatchdogSimulationReport>;
}
