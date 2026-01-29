/// Thin CLI adapter for fastq.correct.
///
/// # Errors
/// Propagates planning or execution errors from the router.
pub fn bench_fastq_correct<S: ::std::hash::BuildHasher>(
    catalog: &std::collections::HashMap<String, bijux_engine::api::ToolImageSpec, S>,
    platform: &bijux_engine::api::PlatformSpec,
    runner_override: Option<bijux_engine::api::RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqCorrectArgs,
) -> anyhow::Result<crate::fastq_router::BenchOutcome<bijux_analyze::FastqCorrectMetrics>> {
    crate::fastq_router::bench_fastq_correct(catalog, platform, runner_override, args)
}
