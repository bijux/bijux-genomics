//! Governed BAM benchmark admission.
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;

const BENCHMARK_READINESS_ROOT_RELATIVE: &str = "benchmarks/readiness";

pub(in crate::benchmark::cases) fn local_smoke_case_output_dir(
    repo_root: &Path,
    output_root: &Path,
    sample_id: &str,
    tool_id: &str,
) -> Result<PathBuf> {
    let resolved_root = if output_root.is_absolute() {
        output_root.to_path_buf()
    } else {
        repo_root.join(output_root)
    };
    let readiness_root = repo_root.join(BENCHMARK_READINESS_ROOT_RELATIVE);
    if resolved_root == readiness_root || resolved_root.starts_with(&readiness_root) {
        return Err(anyhow!(
            "local-smoke output_dir must remain disposable under `artifacts/runs/bench/local-smoke` and must not resolve inside `{BENCHMARK_READINESS_ROOT_RELATIVE}`"
        ));
    }
    Ok(output_root.join(sample_id).join(tool_id))
}

pub(in crate::benchmark::cases) fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}

#[cfg(test)]
mod tests {
    use super::local_smoke_case_output_dir;
    use std::path::Path;
    #[test]
    fn local_smoke_case_output_dir_rejects_readiness_root() {
        let error = local_smoke_case_output_dir(
            Path::new("/workspace/repo"),
            Path::new("benchmarks/readiness/local-ready/bam.validate"),
            "core-v1-coordinate-pass",
            "samtools",
        )
        .expect_err("readiness-root smoke outputs must be refused");

        assert!(
            error.to_string().contains("local-smoke output_dir must remain disposable"),
            "unexpected error: {error}"
        );
    }
}
