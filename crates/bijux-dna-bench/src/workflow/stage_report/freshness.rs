//! Owner: bijux-dna-bench
//! Source freshness across immutable generated benchmark publications.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use anyhow::{bail, Result};
use bijux_dna_bench_model::AdmittedBenchmarkStageReport;

pub(in crate::workflow) fn current_report_sources(
    repository_root: &Path,
    reports: &[AdmittedBenchmarkStageReport],
    current_commit: &str,
) -> Result<BTreeSet<String>> {
    let mut sources = BTreeSet::new();
    for report in reports {
        let source = &report.report.tools[0].repository_commit;
        if report_source_is_current(repository_root, source, current_commit)? {
            sources.insert(source.clone());
        }
    }
    Ok(sources)
}

pub(super) fn report_source_is_current(
    repository_root: &Path,
    report_commit: &str,
    current_commit: &str,
) -> Result<bool> {
    let observed_head = bijux_dna_environment::api::repository_head_commit(repository_root)?;
    if observed_head != current_commit {
        bail!("benchmark freshness target differs from repository HEAD");
    }
    if report_commit == current_commit {
        return Ok(true);
    }
    if !bijux_dna_environment::api::repository_commit_is_ancestor_of_head(
        repository_root,
        report_commit,
    )? {
        return Ok(false);
    }
    let changed =
        bijux_dna_environment::api::repository_changed_paths_since(repository_root, report_commit)?;
    Ok(!changed.is_empty() && changed.iter().all(|path| is_governed_publication_path(path)))
}

fn is_governed_publication_path(value: &str) -> bool {
    let path = Path::new(value);
    let mut components = path.components();
    if components.next().and_then(component_text) != Some("benchmarks")
        || components.next().and_then(component_text) != Some("reports")
    {
        return false;
    }
    let Some(file_name) = components.next().and_then(component_text) else {
        return false;
    };
    if components.next().is_some() {
        return false;
    }
    matches!(
        file_name,
        "scientific-benchmark-readiness.json"
            | "scientific-benchmark-readiness.md"
            | "stage-benchmark-coverage.json"
            | "stage-benchmark-coverage.md"
    ) || governed_run_report_name(file_name)
}

fn governed_run_report_name(file_name: &str) -> bool {
    let stem = file_name.strip_suffix(".json").or_else(|| file_name.strip_suffix(".md"));
    stem.is_some_and(|stem| {
        stem.len() > 4
            && stem.starts_with("run-")
            && stem.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

fn component_text(component: Component<'_>) -> Option<&str> {
    match component {
        Component::Normal(value) => value.to_str(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use super::{is_governed_publication_path, report_source_is_current};

    #[test]
    fn freshness_exempts_only_governed_report_publications() {
        for path in [
            "benchmarks/reports/run-2004-bam-mapping-summary.json",
            "benchmarks/reports/run-2004-bam-mapping-summary.md",
            "benchmarks/reports/stage-benchmark-coverage.json",
            "benchmarks/reports/scientific-benchmark-readiness.md",
        ] {
            assert!(is_governed_publication_path(path));
        }
        for path in [
            "benchmarks/contracts/stage-campaigns.toml",
            "benchmarks/reports/nested/run-2004.json",
            "benchmarks/reports/run-2004.schema.json",
            "containers/contracts/catalog.toml",
            "crates/bijux-dna-bench/src/lib.rs",
        ] {
            assert!(!is_governed_publication_path(path));
        }
    }

    #[test]
    fn report_publication_descendants_remain_current_but_source_changes_do_not(
    ) -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("benchmark-report-freshness")?;
        let root = temporary.path();
        git(root, &["init", "--quiet"])?;
        git(root, &["config", "user.email", "tests@bijux.invalid"])?;
        git(root, &["config", "user.name", "Bijux Tests"])?;
        std::fs::write(root.join("authority.toml"), b"version = 1\n")?;
        git(root, &["add", "authority.toml"])?;
        git(root, &["commit", "--quiet", "-m", "source authority"])?;
        let source = bijux_dna_environment::api::repository_head_commit(root)?;

        std::fs::create_dir_all(root.join("benchmarks/reports"))?;
        std::fs::write(root.join("benchmarks/reports/run-2004-report.json"), b"{}\n")?;
        git(root, &["add", "benchmarks/reports/run-2004-report.json"])?;
        git(root, &["commit", "--quiet", "-m", "publish report"])?;
        let publication = bijux_dna_environment::api::repository_head_commit(root)?;
        assert!(report_source_is_current(root, &source, &publication)?);

        std::fs::write(root.join("authority.toml"), b"version = 2\n")?;
        git(root, &["add", "authority.toml"])?;
        git(root, &["commit", "--quiet", "-m", "change authority"])?;
        let changed = bijux_dna_environment::api::repository_head_commit(root)?;
        assert!(!report_source_is_current(root, &source, &changed)?);
        Ok(())
    }

    fn git(root: &Path, arguments: &[&str]) -> anyhow::Result<()> {
        let output = Command::new("git").arg("-C").arg(root).args(arguments).output()?;
        anyhow::ensure!(
            output.status.success(),
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}
