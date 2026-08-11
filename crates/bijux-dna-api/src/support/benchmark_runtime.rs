use anyhow::Result;

pub fn ensure_bench_runner(
    platform: &bijux_dna_containers::api::PlatformSpec,
    runner_override: Option<bijux_dna_containers::api::RuntimeKind>,
) -> Result<bijux_dna_containers::api::RuntimeKind> {
    bijux_dna_bench::resolve_benchmark_runtime(platform, runner_override)
}
