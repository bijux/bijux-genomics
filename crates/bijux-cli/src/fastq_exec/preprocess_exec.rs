/// Thin CLI adapter for fastq.preprocess.
///
/// # Errors
/// Propagates planning or execution errors from the router.
pub fn bench_fastq_preprocess<S: ::std::hash::BuildHasher>(
    catalog: &std::collections::HashMap<String, bijux_engine::api::ToolImageSpec, S>,
    platform: &bijux_engine::api::PlatformSpec,
    runner_override: Option<bijux_engine::api::RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> anyhow::Result<()> {
    crate::fastq_router::bench_fastq_preprocess(catalog, platform, runner_override, args)
}

/// Execute the preprocess pipeline.
///
/// # Errors
/// Propagates planning or execution errors from the router.
#[allow(dead_code)]
pub fn fastq_preprocess_run<S: ::std::hash::BuildHasher>(
    catalog: &std::collections::HashMap<String, bijux_engine::api::ToolImageSpec, S>,
    platform: &bijux_engine::api::PlatformSpec,
    runner_override: Option<bijux_engine::api::RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> anyhow::Result<()> {
    crate::fastq_router::fastq_preprocess_run(catalog, platform, runner_override, args)
}
