use anyhow::{anyhow, Result};

pub fn ensure_bench_runner(
    platform: &bijux_dna_environment::api::PlatformSpec,
    runner_override: Option<bijux_dna_environment::api::RuntimeKind>,
) -> Result<bijux_dna_environment::api::RuntimeKind> {
    let runner = runner_override.unwrap_or(platform.runner);
    if !matches!(
        runner,
        bijux_dna_environment::api::RuntimeKind::Docker
            | bijux_dna_environment::api::RuntimeKind::Apptainer
            | bijux_dna_environment::api::RuntimeKind::Singularity
    ) {
        return Err(anyhow!("benchmarking does not support runner {runner}"));
    }
    Ok(runner)
}
