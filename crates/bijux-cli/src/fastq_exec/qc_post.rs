/// Thin CLI adapter for `fastq.qc_post`.
///
/// # Errors
/// Propagates planning or execution errors from the router.
pub fn bench_fastq_qc_post<S: ::std::hash::BuildHasher>(
    catalog: &std::collections::HashMap<String, bijux_engine::api::ToolImageSpec, S>,
    platform: &bijux_engine::api::PlatformSpec,
    runner_override: Option<bijux_engine::api::RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqQcPostArgs,
) -> anyhow::Result<crate::fastq_router::BenchOutcome<bijux_analyze::FastqQcPostMetrics>> {
    crate::fastq_router::bench_fastq_qc_post(catalog, platform, runner_override, args)
}
