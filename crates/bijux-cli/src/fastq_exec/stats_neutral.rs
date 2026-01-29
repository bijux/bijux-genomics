/// Thin CLI adapter for `fastq.stats_neutral`.
///
/// # Errors
/// Propagates planning or execution errors from the router.
pub fn bench_fastq_stats_neutral<S: ::std::hash::BuildHasher>(
    catalog: &std::collections::HashMap<String, bijux_engine::api::ToolImageSpec, S>,
    platform: &bijux_engine::api::PlatformSpec,
    runner_override: Option<bijux_engine::api::RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqStatsArgs,
) -> anyhow::Result<crate::fastq_router::BenchOutcome<bijux_analyze::FastqStatsMetrics>> {
    crate::fastq_router::bench_fastq_stats_neutral(catalog, platform, runner_override, args)
}
