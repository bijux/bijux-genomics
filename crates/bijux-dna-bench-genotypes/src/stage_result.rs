//! Owner: bijux-dna-bench-genotypes
//! Persisted stage-result contracts belong to the benchmark model crate.
use std::path::Path;

use anyhow::Result;

use bijux_dna_bench_model::validate_benchmark_stage_result_manifest;
pub(crate) use bijux_dna_bench_model::{
    BenchmarkStageResultCommand as BenchStageResultCommandV1,
    BenchmarkStageResultManifest as BenchStageResultManifestV1,
    BenchmarkStageResultOutput as BenchStageResultOutputV1,
    BenchmarkStageResultResourceMetricSource as BenchStageResultResourceMetricSource,
    BenchmarkStageResultResourceMetrics as BenchStageResultResourceMetricsV1,
    BenchmarkStageResultRuntime as BenchStageResultRuntimeV1,
    BenchmarkStageResultStatus as BenchStageResultStatus,
    BenchmarkStageResultTool as BenchStageResultToolV1,
    BENCHMARK_STAGE_RESULT_SCHEMA_V2 as BENCH_STAGE_RESULT_SCHEMA_VERSION,
};

pub(crate) fn validate_stage_result_manifest(manifest: &BenchStageResultManifestV1) -> Result<()> {
    validate_benchmark_stage_result_manifest(manifest).map_err(Into::into)
}

pub(crate) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}
