use super::LocalReadinessAdapter;
use anyhow::Result;
use bijux_dna_api::v1::api::bench::{
    BenchmarkLocalStageCommandManifest, BenchmarkLocalStageFakeFailureManifest,
    BenchmarkLocalStageFakeRunManifest, BenchmarkLocalStageManifestCompletionReport,
    BenchmarkLocalStageOutputCompletionReport, BenchmarkLocalStageRuntimeMetricsReport,
    BenchmarkLocalSummaryReport, BenchmarkStageResultManifest,
};
use bijux_dna_bench_hpc::readiness::ports::{HarnessEvidence, ToolComparisonObservation};
use std::path::{Path, PathBuf};
impl HarnessEvidence for LocalReadinessAdapter {
    fn render_local_stage_commands(
        &self,
        repo_root: &Path,
        output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageCommandManifest> {
        crate::commands::benchmark::local_stage_commands::render_local_stage_commands(
            repo_root,
            output_path,
        )
    }
    fn fake_run_local_stage_commands(
        &self,
        repo_root: &Path,
        output_root: PathBuf,
    ) -> Result<BenchmarkLocalStageFakeRunManifest> {
        crate::commands::benchmark::local_stage_fake_runs::fake_run_local_stage_commands(
            repo_root,
            output_root,
        )
    }
    fn fake_run_local_stage_failures(
        &self,
        repo_root: &Path,
        output_root: PathBuf,
        stage_ids: &[String],
        exit_code: i32,
    ) -> Result<BenchmarkLocalStageFakeFailureManifest> {
        crate::commands::benchmark::local_stage_fake_runs::fake_run_local_stage_failures(
            repo_root,
            output_root,
            stage_ids,
            exit_code,
        )
    }
    fn check_local_stage_output_completion(
        &self,
        repo_root: &Path,
        fake_run_root: PathBuf,
        report_output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageOutputCompletionReport> {
        crate::commands::benchmark::local_stage_output_completion::check_local_stage_output_completion(repo_root, fake_run_root, report_output_path)
    }
    fn check_local_stage_manifest_completion(
        &self,
        repo_root: &Path,
        fake_run_root: PathBuf,
        report_output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageManifestCompletionReport> {
        crate::commands::benchmark::local_stage_manifest_completion::check_local_stage_manifest_completion(repo_root, fake_run_root, report_output_path)
    }
    fn load_validated_stage_result_manifest_path(
        &self,
        manifest_path: &Path,
    ) -> Result<BenchmarkStageResultManifest> {
        crate::commands::benchmark::local_stage_result_manifest::load_validated_stage_result_manifest_path(manifest_path)
    }
    fn collect_local_stage_runtime_metrics(
        &self,
        repo_root: &Path,
        fake_run_root: PathBuf,
        report_output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageRuntimeMetricsReport> {
        crate::commands::benchmark::local_stage_runtime_metrics::collect_local_stage_runtime_metrics(
            repo_root,
            fake_run_root,
            report_output_path,
        )
    }
    fn render_local_tool_comparison_template(
        &self,
        repo_root: &Path,
        fake_run_root: PathBuf,
        output_path: PathBuf,
    ) -> Result<ToolComparisonObservation> {
        let report = crate::commands::benchmark::local_tool_comparison_template::render_local_tool_comparison_template(repo_root, fake_run_root, output_path)?;
        Ok(ToolComparisonObservation {
            tsv_output_path: report.tsv_output_path,
            row_count: report.row_count,
            observed_row_count: report.rows.len(),
        })
    }
    fn render_local_benchmark_summary(
        &self,
        repo_root: &Path,
        fake_run_root: PathBuf,
        json_output_path: PathBuf,
        markdown_output_path: PathBuf,
    ) -> Result<BenchmarkLocalSummaryReport> {
        crate::commands::benchmark::local_benchmark_summary::render_local_benchmark_summary(
            repo_root,
            fake_run_root,
            json_output_path,
            markdown_output_path,
        )
    }
}
