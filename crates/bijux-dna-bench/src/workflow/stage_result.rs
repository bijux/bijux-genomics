//! Owner: bijux-dna-bench
//! Repository-backed loading and validation of benchmark stage execution results.

use std::path::Path;

use anyhow::Result;
use bijux_dna_bench_model::{
    validate_benchmark_stage_result_validation_report, BenchmarkStageResultValidationReport,
    BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1,
};

pub use crate::repo::stage_result::load_benchmark_stage_result_manifest;

/// Validate one stage execution result and project its stable operator report.
///
/// # Errors
/// Returns an error when the result is invalid or unavailable.
pub fn validate_benchmark_stage_result_path(
    repository_root: &Path,
    manifest_path: &Path,
) -> Result<BenchmarkStageResultValidationReport> {
    let manifest = load_benchmark_stage_result_manifest(manifest_path)?;
    let report = BenchmarkStageResultValidationReport {
        schema_version: BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1.to_string(),
        manifest_path: manifest_path
            .strip_prefix(repository_root)
            .unwrap_or(manifest_path)
            .display()
            .to_string(),
        stage_id: manifest.stage_id,
        tool_id: manifest.tool.id,
        output_count: manifest.outputs.len(),
        status: manifest.runtime.status,
        valid: true,
    };
    validate_benchmark_stage_result_validation_report(&report)?;
    Ok(report)
}
