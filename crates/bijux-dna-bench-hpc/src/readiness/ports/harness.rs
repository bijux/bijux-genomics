use super::observations::ToolComparisonObservation;
use anyhow::Result;
use bijux_dna_bench_model::{
    BenchmarkLocalStageCommandManifest, BenchmarkLocalStageFakeFailureManifest,
    BenchmarkLocalStageFakeRunManifest, BenchmarkLocalStageManifestCompletionReport,
    BenchmarkLocalStageOutputCompletionReport, BenchmarkLocalStageRuntimeMetricsReport,
    BenchmarkLocalSummaryReport, BenchmarkStageResultManifest,
};
use std::path::{Path, PathBuf};
/// Supply typed local harness evidence; provider failures remain explicit readiness diagnostics.
pub trait HarnessEvidence {
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn render_local_stage_commands(
        &self,
        repo_root: &Path,
        output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageCommandManifest>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn fake_run_local_stage_commands(
        &self,
        repo_root: &Path,
        output_root: PathBuf,
    ) -> Result<BenchmarkLocalStageFakeRunManifest>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn fake_run_local_stage_failures(
        &self,
        repo_root: &Path,
        output_root: PathBuf,
        stage_ids: &[String],
        exit_code: i32,
    ) -> Result<BenchmarkLocalStageFakeFailureManifest>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn check_local_stage_output_completion(
        &self,
        repo_root: &Path,
        fake_run_root: PathBuf,
        report_output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageOutputCompletionReport>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn check_local_stage_manifest_completion(
        &self,
        repo_root: &Path,
        fake_run_root: PathBuf,
        report_output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageManifestCompletionReport>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn load_validated_stage_result_manifest_path(
        &self,
        manifest_path: &Path,
    ) -> Result<BenchmarkStageResultManifest>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn collect_local_stage_runtime_metrics(
        &self,
        repo_root: &Path,
        fake_run_root: PathBuf,
        report_output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageRuntimeMetricsReport>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn render_local_tool_comparison_template(
        &self,
        repo_root: &Path,
        fake_run_root: PathBuf,
        output_path: PathBuf,
    ) -> Result<ToolComparisonObservation>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn render_local_benchmark_summary(
        &self,
        repo_root: &Path,
        fake_run_root: PathBuf,
        json_output_path: PathBuf,
        markdown_output_path: PathBuf,
    ) -> Result<BenchmarkLocalSummaryReport>;
}
