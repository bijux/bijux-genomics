//! Admission of disposable FASTQ case output paths.
use anyhow::{ensure, Context, Result};
use std::path::{Component, Path, PathBuf};

const LOCAL_CASE_OUTPUT_ROOT: &str = "artifacts/runs/bench/local-smoke";

pub(in crate::planner::benchmark::cases) fn local_smoke_case_output_dir(
    repo_root: &Path,
    output_root: &Path,
    sample_id: &str,
    tool_id: &str,
) -> Result<PathBuf> {
    ensure!(
        !output_root.components().any(|component| matches!(component, Component::ParentDir)),
        "local-smoke output_dir must not contain parent traversal"
    );
    let relative_root = if output_root.is_absolute() {
        output_root
            .strip_prefix(repo_root)
            .context("local-smoke absolute output_dir must remain inside the repository")?
    } else {
        output_root
    };
    ensure!(
        relative_root.starts_with(LOCAL_CASE_OUTPUT_ROOT),
        "local-smoke output_dir must remain disposable under `{LOCAL_CASE_OUTPUT_ROOT}`"
    );
    admit_case_component(sample_id, "sample_id")?;
    admit_case_component(tool_id, "tool_id")?;
    Ok(output_root.join(sample_id).join(tool_id))
}

fn admit_case_component(value: &str, label: &str) -> Result<()> {
    let mut components = Path::new(value).components();
    ensure!(
        !value.trim().is_empty()
            && !value.contains(['/', '\\', '\0'])
            && matches!(components.next(), Some(Component::Normal(_)))
            && components.next().is_none(),
        "local-smoke {label} must be one non-empty path component"
    );
    Ok(())
}

pub(in crate::planner::benchmark::cases) fn admit_unique_case_samples<'a>(
    stage_id: &str,
    sample_ids: impl IntoIterator<Item = &'a str>,
) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for sample_id in sample_ids {
        ensure!(!sample_id.trim().is_empty(), "local-smoke {stage_id} sample_id must not be empty");
        ensure!(seen.insert(sample_id), "duplicate local-smoke {stage_id} sample_id `{sample_id}`");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../tests/contracts/benchmark/case_output_admission.rs"]
mod contract_tests;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::local_smoke_case_output_dir;

    #[test]
    fn local_smoke_case_output_dir_rejects_readiness_root() {
        let result = local_smoke_case_output_dir(
            Path::new("/workspace/repo"),
            Path::new("benchmarks/readiness/local-ready/fastq.validate_reads"),
            "toy-se",
            "fastqvalidator",
        );
        let error = match result {
            Ok(path) => panic!("readiness-root smoke outputs must be refused, got {path:?}"),
            Err(error) => error,
        };

        assert!(
            error.to_string().contains("local-smoke output_dir must remain disposable"),
            "unexpected error: {error}"
        );
    }
}
