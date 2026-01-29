/// Thin CLI adapter for fastq.trim.
///
/// # Errors
/// Propagates planning or execution errors from the router.
pub fn bench_fastq_trim<S: ::std::hash::BuildHasher>(
    catalog: &std::collections::HashMap<String, bijux_engine::api::ToolImageSpec, S>,
    platform: &bijux_engine::api::PlatformSpec,
    runner_override: Option<bijux_engine::api::RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqTrimArgs,
) -> anyhow::Result<crate::fastq_router::BenchOutcome<bijux_analyze::FastqTrimMetrics>> {
    crate::fastq_router::bench_fastq_trim(catalog, platform, runner_override, args)
}
