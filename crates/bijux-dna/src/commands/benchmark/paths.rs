use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use crate::commands::cli::parse;
use crate::commands::cli::render;

const BENCHMARK_PATHS_VALIDATE_SCHEMA_VERSION: &str = "bijux.bench.paths_validate.v1";
pub(crate) const DEFAULT_BENCHMARK_PATHS_VALIDATE_PATH: &str =
    "target/bench-readiness/benchmark-paths-validation.json";

const REQUIRED_BENCHMARK_ROOTS: &[BenchmarkRootContract] = &[
    BenchmarkRootContract { relative_path: "benchmarks", marker_path: "benchmarks/README.md" },
    BenchmarkRootContract {
        relative_path: "benchmarks/configs",
        marker_path: "benchmarks/configs/README.md",
    },
    BenchmarkRootContract {
        relative_path: "benchmarks/schemas",
        marker_path: "benchmarks/schemas/README.md",
    },
    BenchmarkRootContract {
        relative_path: "benchmarks/tests",
        marker_path: "benchmarks/tests/README.md",
    },
    BenchmarkRootContract {
        relative_path: "benchmarks/readiness",
        marker_path: "benchmarks/readiness/README.md",
    },
];

#[derive(Debug, Clone, Copy)]
struct BenchmarkRootContract {
    relative_path: &'static str,
    marker_path: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchmarkRootStatus {
    pub(crate) relative_path: String,
    pub(crate) marker_path: String,
    pub(crate) exists: bool,
    pub(crate) marker_exists: bool,
    pub(crate) ignored_by_git: bool,
    pub(crate) marker_tracked_by_git: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchmarkPathViolation {
    pub(crate) relative_path: String,
    pub(crate) violation_type: String,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchmarkPathsValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) repo_root: String,
    pub(crate) strict: bool,
    pub(crate) root_count: usize,
    pub(crate) existing_root_count: usize,
    pub(crate) tracked_marker_count: usize,
    pub(crate) ignored_root_count: usize,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) roots: Vec<BenchmarkRootStatus>,
    pub(crate) violations: Vec<BenchmarkPathViolation>,
}

pub(crate) fn run_benchmark_paths_validate_command(
    cwd: &Path,
    args: &parse::BenchPathsValidateArgs,
) -> Result<()> {
    let report = validate_benchmark_paths(
        cwd,
        PathBuf::from(DEFAULT_BENCHMARK_PATHS_VALIDATE_PATH),
        args.strict,
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn validate_benchmark_paths(
    repo_root: &Path,
    output_path: PathBuf,
    strict: bool,
) -> Result<BenchmarkPathsValidationReport> {
    let absolute_output_path = repo_root.join(&output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let roots = REQUIRED_BENCHMARK_ROOTS
        .iter()
        .map(|contract| benchmark_root_status(repo_root, contract))
        .collect::<Result<Vec<_>>>()?;
    let violations = collect_path_violations(&roots);
    let existing_root_count = roots.iter().filter(|root| root.exists).count();
    let tracked_marker_count = roots.iter().filter(|root| root.marker_tracked_by_git).count();
    let ignored_root_count = roots.iter().filter(|root| root.ignored_by_git).count();
    let report = BenchmarkPathsValidationReport {
        schema_version: BENCHMARK_PATHS_VALIDATE_SCHEMA_VERSION,
        output_path: output_path.display().to_string(),
        repo_root: repo_root.display().to_string(),
        strict,
        root_count: roots.len(),
        existing_root_count,
        tracked_marker_count,
        ignored_root_count,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        roots,
        violations,
    };

    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;

    if strict && !report.ok {
        let summary = report
            .violations
            .iter()
            .map(|violation| {
                format!(
                    "{} {} ({})",
                    violation.relative_path, violation.violation_type, violation.detail
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        bail!("benchmark path validation found {} violation(s): {summary}", report.violation_count);
    }

    Ok(report)
}

fn benchmark_root_status(
    repo_root: &Path,
    contract: &BenchmarkRootContract,
) -> Result<BenchmarkRootStatus> {
    let root_path = repo_root.join(contract.relative_path);
    let marker_path = repo_root.join(contract.marker_path);
    Ok(BenchmarkRootStatus {
        relative_path: contract.relative_path.to_string(),
        marker_path: contract.marker_path.to_string(),
        exists: root_path.is_dir(),
        marker_exists: marker_path.is_file(),
        ignored_by_git: git_check_ignored(repo_root, contract.relative_path)?
            || git_check_ignored(repo_root, contract.marker_path)?,
        marker_tracked_by_git: git_check_tracked(repo_root, contract.marker_path)?,
    })
}

fn collect_path_violations(roots: &[BenchmarkRootStatus]) -> Vec<BenchmarkPathViolation> {
    let mut violations = Vec::new();
    for root in roots {
        if !root.exists {
            violations.push(BenchmarkPathViolation {
                relative_path: root.relative_path.clone(),
                violation_type: "missing_root".to_string(),
                detail: "required benchmark root directory is absent".to_string(),
            });
        }
        if !root.marker_exists {
            violations.push(BenchmarkPathViolation {
                relative_path: root.relative_path.clone(),
                violation_type: "missing_marker".to_string(),
                detail: format!("missing tracked marker {}", root.marker_path),
            });
        }
        if root.ignored_by_git {
            violations.push(BenchmarkPathViolation {
                relative_path: root.relative_path.clone(),
                violation_type: "ignored_by_git".to_string(),
                detail: "benchmark root or marker is ignored by git".to_string(),
            });
        }
        if !root.marker_tracked_by_git {
            violations.push(BenchmarkPathViolation {
                relative_path: root.relative_path.clone(),
                violation_type: "untracked_marker".to_string(),
                detail: format!("git does not track {}", root.marker_path),
            });
        }
    }
    violations
}

fn git_check_ignored(repo_root: &Path, relative_path: &str) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["check-ignore", "-q", "--no-index", relative_path])
        .output()
        .with_context(|| format!("run git check-ignore for {relative_path}"))?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        Some(code) => Err(anyhow!(
            "git check-ignore returned {code} for {relative_path}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        None => {
            Err(anyhow!("git check-ignore terminated without an exit code for {relative_path}"))
        }
    }
}

fn git_check_tracked(repo_root: &Path, relative_path: &str) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["ls-files", "--error-unmatch", relative_path])
        .output()
        .with_context(|| format!("run git ls-files for {relative_path}"))?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        Some(code) => Err(anyhow!(
            "git ls-files returned {code} for {relative_path}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        None => Err(anyhow!("git ls-files terminated without an exit code for {relative_path}")),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        collect_path_violations, validate_benchmark_paths, BenchmarkRootStatus,
        DEFAULT_BENCHMARK_PATHS_VALIDATE_PATH,
    };
    use std::path::Path;

    fn write_text(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(path, content).expect("write text");
    }

    fn init_repo(root: &Path) {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["init", "-q"])
            .output()
            .expect("git init");
        assert!(
            output.status.success(),
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["config", "user.email", "benchmarks@example.test"])
            .output()
            .expect("git config user.email");
        assert!(output.status.success());
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["config", "user.name", "benchmarks"])
            .output()
            .expect("git config user.name");
        assert!(output.status.success());
    }

    fn stage_all(root: &Path) {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["add", "benchmarks", ".gitignore"])
            .output()
            .expect("git add");
        assert!(
            output.status.success(),
            "git add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn write_benchmark_root(root: &Path) {
        write_text(&root.join("benchmarks/README.md"), "# Benchmarks\n");
        write_text(&root.join("benchmarks/configs/README.md"), "# Benchmark Configs\n");
        write_text(&root.join("benchmarks/schemas/README.md"), "# Benchmark Schemas\n");
        write_text(&root.join("benchmarks/tests/README.md"), "# Benchmark Tests\n");
        write_text(&root.join("benchmarks/readiness/README.md"), "# Benchmark Readiness\n");
    }

    #[test]
    fn benchmark_path_validation_accepts_tracked_root_markers() {
        let temp = tempfile::tempdir().expect("tempdir");
        init_repo(temp.path());
        write_benchmark_root(temp.path());
        write_text(&temp.path().join(".gitignore"), "");
        stage_all(temp.path());

        let report = validate_benchmark_paths(
            temp.path(),
            std::path::PathBuf::from(DEFAULT_BENCHMARK_PATHS_VALIDATE_PATH),
            true,
        )
        .expect("validate benchmark paths");

        assert!(report.ok);
        assert_eq!(report.root_count, 5);
        assert_eq!(report.existing_root_count, 5);
        assert_eq!(report.tracked_marker_count, 5);
        assert_eq!(report.ignored_root_count, 0);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn benchmark_path_validation_rejects_ignored_roots_in_strict_mode() {
        let temp = tempfile::tempdir().expect("tempdir");
        init_repo(temp.path());
        write_benchmark_root(temp.path());
        write_text(&temp.path().join(".gitignore"), "benchmarks/\n");

        let err = validate_benchmark_paths(
            temp.path(),
            std::path::PathBuf::from(DEFAULT_BENCHMARK_PATHS_VALIDATE_PATH),
            true,
        )
        .expect_err("strict validation should fail");
        assert!(err.to_string().contains("ignored_by_git"));
    }

    #[test]
    fn benchmark_path_violation_collection_detects_missing_and_untracked_markers() {
        let roots = vec![BenchmarkRootStatus {
            relative_path: "benchmarks/configs".to_string(),
            marker_path: "benchmarks/configs/README.md".to_string(),
            exists: true,
            marker_exists: false,
            ignored_by_git: false,
            marker_tracked_by_git: false,
        }];
        let violations = collect_path_violations(&roots);
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].violation_type, "missing_marker");
        assert_eq!(violations[1].violation_type, "untracked_marker");
    }
}
