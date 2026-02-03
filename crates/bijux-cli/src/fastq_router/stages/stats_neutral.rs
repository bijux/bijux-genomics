use std::collections::HashMap;

use anyhow::Result;
use bijux_engine::api::{PlatformSpec, RunnerKind, ToolImageSpec};

use crate::fastq_router::BenchOutcome;

/// Run the stats-neutral pipeline.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_stats_neutral<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqStatsArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqStatsMetrics>> {
    crate::fastq_stats_neutral::bench_fastq_stats_neutral(catalog, platform, runner_override, args)
}
