use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::Serialize;

use crate::commands::cli::BenchRepoChecksArgs;

const EXCLUDED_TOOLING_PATHS: &[&str] = &[];
const LOCAL_OPERATOR_PATH_PREFIX: &str = "/Users/bijan/";
const REMOTE_OPERATOR_PATH_PREFIX: &str = "/home/bijan/";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RepoCheckViolation {
    issue_id: String,
    path: String,
    line: usize,
    literal: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RepoChecksReport {
    check_count: usize,
    violation_count: usize,
    violations: Vec<RepoCheckViolation>,
}

pub(crate) fn run_benchmark_repo_checks_command(
    cwd: &Path,
    args: &BenchRepoChecksArgs,
) -> Result<()> {
    let repo_root = absolutize(cwd, &args.repo_root);
    let report = audit_repo_checks(&repo_root)?;
    if let Some(json_out) = args.json_out.as_deref() {
        let json_path = absolutize(cwd, json_out);
        if let Some(parent) = json_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(
            &json_path,
            format!("{}\n", serde_json::to_string_pretty(&report)?),
        )
        .with_context(|| format!("write {}", json_path.display()))?;
    }
    println!("{}", serde_json::to_string_pretty(&report)?);
    fail_on_repo_check_violations(&report)
}

pub(crate) fn audit_repo_checks(repo_root: &Path) -> Result<RepoChecksReport> {
    let tooling_paths = benchmark_tooling_paths(repo_root)?;
    let contract_paths = benchmark_contract_paths(repo_root)?;
    let mut path_scan_paths = tooling_paths.clone();
    path_scan_paths.extend(contract_paths);
    path_scan_paths.sort();
    path_scan_paths.dedup();

    let mut violations = literal_matches(
        &path_scan_paths,
        repo_root,
        LOCAL_OPERATOR_PATH_PREFIX,
        "hardcoded-local-operator-path",
    )?;
    violations.extend(literal_matches(
        &path_scan_paths,
        repo_root,
        REMOTE_OPERATOR_PATH_PREFIX,
        "hardcoded-remote-operator-path",
    )?);
    violations.extend(regex_matches(
        &tooling_paths,
        repo_root,
        "hardcoded-ssh-host-alias",
        &[
            Regex::new(r#"["']lunarc:[^"']*["']"#).expect("host alias regex"),
            Regex::new(r#"ssh\s+['"]?lunarc(["'\s]|$)"#).expect("ssh lunarc regex"),
            Regex::new(r#"hostname[^\n]*["']lunarc["']"#).expect("hostname lunarc regex"),
        ],
    )?);

    Ok(RepoChecksReport {
        check_count: 3,
        violation_count: violations.len(),
        violations,
    })
}

pub(crate) fn fail_on_repo_check_violations(report: &RepoChecksReport) -> Result<()> {
    if report.violations.is_empty() {
        return Ok(());
    }
    let details = report
        .violations
        .iter()
        .map(|violation| {
            format!(
                "{}:{}: {} {}",
                violation.path, violation.line, violation.issue_id, violation.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Err(anyhow!(
        "benchmark repo checks found {} violation(s):\n{}",
        report.violation_count,
        details
    ))
}

fn benchmark_tooling_paths(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let makes_bin = repo_root.join("makes/bin");
    if makes_bin.is_dir() {
        for entry in makes_bin
            .read_dir()
            .with_context(|| format!("read {}", makes_bin.display()))?
        {
            let entry = entry.with_context(|| format!("read {}", makes_bin.display()))?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "py") {
                let relative = relative_to_root(repo_root, &path)?;
                if !EXCLUDED_TOOLING_PATHS.iter().any(|item| *item == relative) {
                    paths.push(path);
                }
            }
        }
    }

    let makes_root = repo_root.join("makes");
    if makes_root.is_dir() {
        for entry in makes_root
            .read_dir()
            .with_context(|| format!("read {}", makes_root.display()))?
        {
            let entry = entry.with_context(|| format!("read {}", makes_root.display()))?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "mk") {
                paths.push(path);
            }
        }
    }

    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn benchmark_contract_paths(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let bench_config_root = repo_root.join("configs/bench");
    if bench_config_root.is_dir() {
        for entry in bench_config_root
            .read_dir()
            .with_context(|| format!("read {}", bench_config_root.display()))?
        {
            let entry = entry.with_context(|| format!("read {}", bench_config_root.display()))?;
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| matches!(ext, "toml" | "md"))
            {
                paths.push(path);
            }
        }
    }

    let docs_root = repo_root.join("docs/benchmark");
    if docs_root.is_dir() {
        for entry in walkdir::WalkDir::new(&docs_root) {
            let entry = entry.with_context(|| format!("walk {}", docs_root.display()))?;
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| matches!(ext, "md" | "json" | "csv"))
            {
                paths.push(path.to_path_buf());
            }
        }
    }

    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn literal_matches(
    paths: &[PathBuf],
    repo_root: &Path,
    literal: &str,
    issue_id: &str,
) -> Result<Vec<RepoCheckViolation>> {
    let mut matches = Vec::new();
    for path in paths {
        for (line_number, line) in read_lines(path)?.iter().enumerate() {
            if !line.contains(literal) {
                continue;
            }
            matches.push(RepoCheckViolation {
                issue_id: issue_id.to_string(),
                path: relative_to_root(repo_root, path)?.to_string(),
                line: line_number + 1,
                literal: literal.to_string(),
                content: line.trim().to_string(),
            });
        }
    }
    Ok(matches)
}

fn regex_matches(
    paths: &[PathBuf],
    repo_root: &Path,
    issue_id: &str,
    patterns: &[Regex],
) -> Result<Vec<RepoCheckViolation>> {
    let mut matches = Vec::new();
    for path in paths {
        for (line_number, line) in read_lines(path)?.iter().enumerate() {
            if !patterns.iter().any(|pattern| pattern.is_match(line)) {
                continue;
            }
            matches.push(RepoCheckViolation {
                issue_id: issue_id.to_string(),
                path: relative_to_root(repo_root, path)?.to_string(),
                line: line_number + 1,
                literal: "lunarc host literal".to_string(),
                content: line.trim().to_string(),
            });
        }
    }
    Ok(matches)
}

fn read_lines(path: &Path) -> Result<Vec<String>> {
    Ok(fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?
        .lines()
        .map(ToOwned::to_owned)
        .collect())
}

fn relative_to_root<'a>(repo_root: &Path, path: &'a Path) -> Result<&'a str> {
    path.strip_prefix(repo_root)
        .with_context(|| format!("strip {} from {}", repo_root.display(), path.display()))?
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 path {}", path.display()))
}

fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::audit_repo_checks;
    use std::fs;

    #[test]
    fn repo_checks_flag_hardcoded_local_operator_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path();
        let script_path = repo_root.join("makes/bin/example.py");
        fs::create_dir_all(script_path.parent().expect("script dir")).expect("create script dir");
        fs::write(
            &script_path,
            "RESULTS_ROOT = \"/Users/bijan/bijux/bijux-dna-results\"\n",
        )
        .expect("write script");

        let report = audit_repo_checks(repo_root).expect("repo checks");
        assert_eq!(report.violation_count, 1);
        assert_eq!(report.violations[0].issue_id, "hardcoded-local-operator-path");
    }

    #[test]
    fn repo_checks_ignore_test_fixture_paths() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path();
        let fixture_path = repo_root.join("makes/bin/test_corpus_01_fastq_benchmarks.py");
        fs::create_dir_all(fixture_path.parent().expect("fixture dir")).expect("create fixture dir");
        fs::write(
            &fixture_path,
            "LOCAL_RESULTS = \"/Users/bijan/bijux/bijux-dna-results\"\n",
        )
        .expect("write fixture");

        let report = audit_repo_checks(repo_root).expect("repo checks");
        assert_eq!(report.violation_count, 0);
    }

    #[test]
    fn repo_checks_flag_hardcoded_lunarc_host_alias() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path();
        let makefile_path = repo_root.join("makes/sync.mk");
        fs::create_dir_all(makefile_path.parent().expect("make dir")).expect("create make dir");
        fs::write(&makefile_path, "SYNC_TARGET := \"lunarc:results-mirror/\"\n")
            .expect("write makefile");

        let report = audit_repo_checks(repo_root).expect("repo checks");
        assert_eq!(report.violation_count, 1);
        assert_eq!(report.violations[0].issue_id, "hardcoded-ssh-host-alias");
    }
}
