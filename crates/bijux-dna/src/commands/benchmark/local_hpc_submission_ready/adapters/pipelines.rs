use super::LocalReadinessAdapter;
use anyhow::Result;
use bijux_dna_api::v1::api::bench::BenchmarkLocalPipelineDagValidationReport;
use bijux_dna_bench_hpc::readiness::ports::PipelineEvidence;
use bijux_dna_bench_hpc::watchdog::{LocalDagWatchdogScenario, LocalDagWatchdogSimulationReport};
use std::path::Path;
impl PipelineEvidence for LocalReadinessAdapter {
    fn validate_pipeline_dag_path(
        &self,
        repo_root: &Path,
        config_path: &Path,
        output_path: &Path,
    ) -> Result<BenchmarkLocalPipelineDagValidationReport> {
        crate::commands::benchmark::local_pipeline_dag::validate_pipeline_dag_path(
            repo_root,
            config_path,
            output_path,
        )
    }
    fn simulate_dag_watchdog_path(
        &self,
        repo_root: &Path,
        scenario: LocalDagWatchdogScenario,
        output_path: &Path,
    ) -> Result<LocalDagWatchdogSimulationReport> {
        crate::commands::benchmark::local_dag_watchdog_simulation::simulate_dag_watchdog_path(
            repo_root,
            scenario,
            output_path,
        )
    }
}
