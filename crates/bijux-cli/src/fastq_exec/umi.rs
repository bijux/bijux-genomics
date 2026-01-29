/// Thin CLI adapter for fastq.umi.
///
/// # Errors
/// Propagates planning or execution errors from the router.
pub fn bench_fastq_umi<S: ::std::hash::BuildHasher>(
    catalog: &std::collections::HashMap<String, bijux_engine::api::ToolImageSpec, S>,
    platform: &bijux_engine::api::PlatformSpec,
    runner_override: Option<bijux_engine::api::RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqUmiArgs,
) -> anyhow::Result<crate::fastq_router::BenchOutcome<bijux_analyze::FastqUmiMetrics>> {
    crate::fastq_router::bench_fastq_umi(catalog, platform, runner_override, args)
}
