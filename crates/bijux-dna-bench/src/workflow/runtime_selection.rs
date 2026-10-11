//! Owner: bijux-dna-bench
//! Benchmark runtime selection policy over runner-supported container backends.

use anyhow::{bail, Result};
use bijux_dna_containers::api::{PlatformSpec, RuntimeKind};

/// Resolve the container runtime used by a benchmark execution.
///
/// # Errors
/// Returns an error when the selected runtime cannot provide the benchmark's container boundary.
pub fn resolve_benchmark_runtime(
    platform: &PlatformSpec,
    runtime_override: Option<RuntimeKind>,
) -> Result<RuntimeKind> {
    let runtime = runtime_override.unwrap_or(platform.runner);
    if matches!(runtime, RuntimeKind::Docker | RuntimeKind::Apptainer | RuntimeKind::Singularity) {
        Ok(runtime)
    } else {
        bail!("benchmarking does not support runner {runtime}")
    }
}

#[cfg(test)]
mod tests {
    use bijux_dna_containers::api::{PlatformSpec, RuntimeKind};

    use super::resolve_benchmark_runtime;

    fn platform(runtime: RuntimeKind) -> PlatformSpec {
        PlatformSpec {
            name: "test-platform".to_string(),
            runner: runtime,
            container_dir: "/containers".into(),
            image_prefix: "registry.example/bijux".to_string(),
            arch: "amd64".to_string(),
        }
    }

    #[test]
    fn platform_runtime_is_used_without_an_override() -> anyhow::Result<()> {
        let platform = platform(RuntimeKind::Apptainer);
        assert_eq!(resolve_benchmark_runtime(&platform, None)?, RuntimeKind::Apptainer);
        Ok(())
    }

    #[test]
    fn explicit_container_runtime_overrides_the_platform() -> anyhow::Result<()> {
        let platform = platform(RuntimeKind::Docker);
        assert_eq!(
            resolve_benchmark_runtime(&platform, Some(RuntimeKind::Singularity))?,
            RuntimeKind::Singularity
        );
        Ok(())
    }

    #[test]
    fn local_runtime_is_rejected_before_benchmark_execution() {
        let error = resolve_benchmark_runtime(&platform(RuntimeKind::Local), None)
            .expect_err("local execution must not bypass the benchmark container boundary");
        assert!(error.to_string().contains("does not support runner local"));
    }
}
