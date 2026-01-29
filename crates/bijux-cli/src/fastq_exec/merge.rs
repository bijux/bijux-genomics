/// Thin CLI adapter for fastq.merge.
///
/// # Errors
/// Propagates planning or execution errors from the router.
pub fn bench_fastq_merge<S: ::std::hash::BuildHasher>(
    catalog: &std::collections::HashMap<String, bijux_engine::api::ToolImageSpec, S>,
    platform: &bijux_engine::api::PlatformSpec,
    runner_override: Option<bijux_engine::api::RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqMergeArgs,
) -> anyhow::Result<crate::fastq_router::BenchOutcome<bijux_analyze::FastqMergeMetrics>> {
    crate::fastq_router::bench_fastq_merge(catalog, platform, runner_override, args)
}
