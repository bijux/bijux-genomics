use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::Serialize;

use crate::commands::cli::BenchRepoChecksArgs;

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
        fs::write(&json_path, format!("{}\n", serde_json::to_string_pretty(&report)?))
            .with_context(|| format!("write {}", json_path.display()))?;
    }
    println!("{}", serde_json::to_string_pretty(&report)?);
    fail_on_repo_check_violations(&report)
}

pub(crate) fn audit_repo_checks(repo_root: &Path) -> Result<RepoChecksReport> {
    let tooling_paths = benchmark_tooling_paths(repo_root)?;
    let contract_paths = benchmark_contract_paths(repo_root)?;
    let crate_paths = benchmark_crate_paths(repo_root)?;
    let mut path_scan_paths = tooling_paths.clone();
    path_scan_paths.extend(contract_paths);
    path_scan_paths.extend(crate_paths.clone());
    path_scan_paths.sort();
    path_scan_paths.dedup();

    let local_user_patterns = [Regex::new(r#"/Users/[^/"'\s]+/"#)
        .map_err(|err| anyhow!("compile local user path regex: {err}"))?];
    let remote_user_patterns = [Regex::new(r#"/home/[^/"'\s]+/"#)
        .map_err(|err| anyhow!("compile remote user path regex: {err}"))?];
    let host_alias_patterns = [
        Regex::new(r#"["']lunarc:[^"']*["']"#)
            .map_err(|err| anyhow!("compile host alias regex: {err}"))?,
        Regex::new(r#"ssh\s+['"]?lunarc(["'\s]|$)"#)
            .map_err(|err| anyhow!("compile ssh lunarc regex: {err}"))?,
        Regex::new(r#"hostname[^\n]*["']lunarc["']"#)
            .map_err(|err| anyhow!("compile hostname lunarc regex: {err}"))?,
    ];

    let mut violations = regex_matches(
        &path_scan_paths,
        repo_root,
        "hardcoded-local-user-path",
        &local_user_patterns,
    )?;
    violations.extend(regex_matches(
        &path_scan_paths,
        repo_root,
        "hardcoded-remote-user-path",
        &remote_user_patterns,
    )?);
    violations.extend(regex_matches(
        &tooling_paths.iter().chain(crate_paths.iter()).cloned().collect::<Vec<_>>(),
        repo_root,
        "hardcoded-ssh-host-alias",
        &host_alias_patterns,
    )?);

    Ok(RepoChecksReport { check_count: 3, violation_count: violations.len(), violations })
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
        for entry in
            makes_bin.read_dir().with_context(|| format!("read {}", makes_bin.display()))?
        {
            let entry = entry.with_context(|| format!("read {}", makes_bin.display()))?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "py") {
                let relative = relative_to_root(repo_root, &path)?;
                if !is_excluded_tooling_path(relative) {
                    paths.push(path);
                }
            }
        }
    }

    let makes_root = repo_root.join("makes");
    if makes_root.is_dir() {
        for entry in
            makes_root.read_dir().with_context(|| format!("read {}", makes_root.display()))?
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

fn is_excluded_tooling_path(relative: &str) -> bool {
    relative.starts_with("makes/bin/test_")
}

fn collect_matching_files_recursive(
    root: &Path,
    predicate: &impl Fn(&Path) -> bool,
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in dir.read_dir().with_context(|| format!("read {}", dir.display()))? {
            let entry = entry.with_context(|| format!("read {}", dir.display()))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if predicate(&path) {
                paths.push(path);
            }
        }
    }
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

    let docs_root = repo_root.join("docs/30-operations/benchmark");
    if docs_root.is_dir() {
        paths.extend(collect_matching_files_recursive(&docs_root, &|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| matches!(ext, "md" | "json" | "csv"))
        })?);
    }

    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn benchmark_crate_paths(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let roots = [
        repo_root.join("crates/bijux-dna/src/commands"),
        repo_root.join("crates/bijux-dna-dev/src/commands"),
        repo_root.join("crates/bijux-dna-api/src/runtime/run"),
        repo_root.join("crates/bijux-dna-stage-contract/src"),
    ];
    let mut paths = Vec::new();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        paths.extend(collect_matching_files_recursive(&root, &|path| {
            path.extension().and_then(|ext| ext.to_str()) == Some("rs")
                && path.file_name().and_then(|name| name.to_str()) != Some("mod.rs")
        })?);
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn regex_matches(
    paths: &[PathBuf],
    repo_root: &Path,
    issue_id: &str,
    patterns: &[Regex],
) -> Result<Vec<RepoCheckViolation>> {
    let mut matches = Vec::new();
    for path in paths {
        for (line_number, line) in repo_check_lines(path)?.iter().enumerate() {
            let Some(literal) = patterns
                .iter()
                .find_map(|pattern| pattern.find(line).map(|matched| matched.as_str().to_string()))
            else {
                continue;
            };
            matches.push(RepoCheckViolation {
                issue_id: issue_id.to_string(),
                path: relative_to_root(repo_root, path)?.to_string(),
                line: line_number + 1,
                literal,
                content: line.trim().to_string(),
            });
        }
    }
    Ok(matches)
}

fn repo_check_lines(path: &Path) -> Result<Vec<String>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
        return Ok(strip_rust_test_modules(&raw));
    }
    Ok(raw.lines().map(ToOwned::to_owned).collect())
}

fn strip_rust_test_modules(raw: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut awaiting_test_module = false;
    let mut skip_depth: usize = 0;

    for raw_line in raw.lines() {
        let trimmed = raw_line.trim();
        if skip_depth > 0 {
            skip_depth = skip_depth
                .saturating_add(raw_line.matches('{').count())
                .saturating_sub(raw_line.matches('}').count());
            continue;
        }
        if trimmed == "#[cfg(test)]" {
            awaiting_test_module = true;
            continue;
        }
        if awaiting_test_module {
            if trimmed.is_empty() || trimmed.starts_with("#[") {
                continue;
            }
            if trimmed.starts_with("mod ") && raw_line.contains('{') {
                skip_depth =
                    raw_line.matches('{').count().saturating_sub(raw_line.matches('}').count());
                awaiting_test_module = false;
                continue;
            }
            awaiting_test_module = false;
        }
        lines.push(raw_line.to_string());
    }
    lines
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
#[allow(clippy::expect_used)]
mod tests {
    use super::{audit_repo_checks, strip_rust_test_modules};
    use std::fs;

    #[test]
    fn repo_checks_flag_hardcoded_local_operator_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path();
        let script_path = repo_root.join("makes/bin/example.py");
        fs::create_dir_all(script_path.parent().expect("script dir")).expect("create script dir");
        fs::write(&script_path, "RESULTS_ROOT = \"/Users/operator/workspace/results\"\n")
            .expect("write script");

        let report = audit_repo_checks(repo_root).expect("repo checks");
        assert_eq!(report.violation_count, 1);
        assert_eq!(report.violations[0].issue_id, "hardcoded-local-user-path");
        assert_eq!(report.violations[0].literal, "/Users/operator/");
    }

    #[test]
    fn repo_checks_ignore_test_fixture_paths() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path();
        let fixture_path = repo_root.join("makes/bin/test_benchmark_fastq_suite.py");
        fs::create_dir_all(fixture_path.parent().expect("fixture dir"))
            .expect("create fixture dir");
        fs::write(&fixture_path, "LOCAL_RESULTS = \"/Users/operator/workspace/results\"\n")
            .expect("write fixture");

        let report = audit_repo_checks(repo_root).expect("repo checks");
        assert_eq!(report.violation_count, 0);
    }

    #[test]
    fn repo_checks_flag_hardcoded_remote_user_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path();
        let config_path = repo_root.join("configs/bench/benchmark.toml");
        fs::create_dir_all(config_path.parent().expect("config dir")).expect("create config dir");
        fs::write(&config_path, "remote_root = \"/home/alice/bijux/results\"\n")
            .expect("write config");

        let report = audit_repo_checks(repo_root).expect("repo checks");
        assert_eq!(report.violation_count, 1);
        assert_eq!(report.violations[0].issue_id, "hardcoded-remote-user-path");
        assert_eq!(report.violations[0].literal, "/home/alice/");
    }

    #[test]
    fn repo_checks_flag_hardcoded_site_host_alias() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path();
        let makefile_path = repo_root.join("makes/sync.mk");
        fs::create_dir_all(makefile_path.parent().expect("make dir")).expect("create make dir");
        fs::write(&makefile_path, "SYNC_TARGET := \"lunarc:results-mirror/\"\n")
            .expect("write makefile");

        let report = audit_repo_checks(repo_root).expect("repo checks");
        assert_eq!(report.violation_count, 1);
        assert_eq!(report.violations[0].issue_id, "hardcoded-ssh-host-alias");
        assert_eq!(report.violations[0].literal, "\"lunarc:results-mirror/\"");
    }

    #[test]
    fn strip_rust_test_modules_excludes_cfg_test_blocks() {
        let stripped = strip_rust_test_modules(
            r#"
fn governed() {
    let host = "cluster";
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    #[test]
    fn fixture() {
        let host = "lunarc";
    }
}
"#,
        );
        let joined = stripped.join("\n");
        assert!(joined.contains("governed"));
        assert!(!joined.contains("lunarc"));
    }
}
