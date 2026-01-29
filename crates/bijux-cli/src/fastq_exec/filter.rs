/// Thin CLI adapter for fastq.filter.
///
/// # Errors
/// Propagates planning or execution errors from the router.
pub fn bench_fastq_filter<S: ::std::hash::BuildHasher>(
    catalog: &std::collections::HashMap<String, bijux_engine::api::ToolImageSpec, S>,
    platform: &bijux_engine::api::PlatformSpec,
    runner_override: Option<bijux_engine::api::RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqFilterArgs,
) -> anyhow::Result<crate::fastq_router::BenchOutcome<bijux_analyze::FastqFilterMetrics>> {
    crate::fastq_router::bench_fastq_filter(catalog, platform, runner_override, args)
}
