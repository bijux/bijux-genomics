/// Thin CLI adapter for `fastq.validate_pre`.
///
/// # Errors
/// Propagates planning or execution errors from the router.
pub fn bench_fastq_validate_pre<S: ::std::hash::BuildHasher>(
    catalog: &std::collections::HashMap<String, bijux_engine::api::ToolImageSpec, S>,
    platform: &bijux_engine::api::PlatformSpec,
    runner_override: Option<bijux_engine::api::RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqValidateArgs,
) -> anyhow::Result<crate::fastq_router::BenchOutcome<bijux_analyze::FastqValidateMetrics>> {
    crate::fastq_router::bench_fastq_validate_pre(catalog, platform, runner_override, args)
}
