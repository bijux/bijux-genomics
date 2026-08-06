use anyhow::{anyhow, Result};

pub fn ensure_bench_runner(
    platform: &bijux_dna_containers::api::PlatformSpec,
    runner_override: Option<bijux_dna_containers::api::RuntimeKind>,
) -> Result<bijux_dna_containers::api::RuntimeKind> {
    let runner = runner_override.unwrap_or(platform.runner);
    if !matches!(
        runner,
        bijux_dna_containers::api::RuntimeKind::Docker
            | bijux_dna_containers::api::RuntimeKind::Apptainer
            | bijux_dna_containers::api::RuntimeKind::Singularity
    ) {
        return Err(anyhow!("benchmarking does not support runner {runner}"));
    }
    Ok(runner)
}
