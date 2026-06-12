use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use crate::commands::cli::parse;
use crate::commands::cli::render;

const BENCHMARK_PATHS_VALIDATE_SCHEMA_VERSION: &str = "bijux.bench.paths_validate.v1";
pub(crate) const DEFAULT_BENCHMARK_PATHS_VALIDATE_PATH: &str =
    "benchmarks/readiness/benchmark-paths-validation.json";
const DISPOSABLE_ROOT_CLEANUP_PROOF_SCHEMA_VERSION: &str =
    "bijux.bench.disposable_root_cleanup_proof.v1";
pub(crate) const DEFAULT_DISPOSABLE_ROOT_CLEANUP_PROOF_PATH: &str =
    "benchmarks/readiness/path-cleanup/DELETE_DISPOSABLE_ROOTS_SAFE.json";
const LEGACY_FIXTURE_WRAPPER_PATH: &str = "tests/fixtures";
const LEGACY_FIXTURE_WRAPPER_TARGET: &str = "../benchmarks/tests/fixtures";
const ROOT_TESTS_README_PATH: &str = "tests/README.md";
const DISPOSABLE_ROOTS: &[&str] = &["target", "artifacts", "runs"];

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
    BenchmarkRootContract {
        relative_path: "benchmarks/readiness/local-ready",
        marker_path: "benchmarks/readiness/local-ready/README.md",
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
    pub(crate) readiness_snapshot_count: usize,
    pub(crate) readiness_json_snapshot_count: usize,
    pub(crate) readiness_tsv_snapshot_count: usize,
    pub(crate) root_tests_regular_file_count: usize,
    pub(crate) root_tests_readme_tracked_by_git: bool,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) legacy_fixture_wrapper: LegacyFixtureWrapperStatus,
    pub(crate) readiness_snapshots: Vec<String>,
    pub(crate) roots: Vec<BenchmarkRootStatus>,
    pub(crate) violations: Vec<BenchmarkPathViolation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DisposableRootDeletionStatus {
    pub(crate) relative_path: String,
    pub(crate) existed_before: bool,
    pub(crate) exists_after: bool,
    pub(crate) removal_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DisposableRootCleanupProofReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) repo_root: String,
    pub(crate) deleted_root_count: usize,
    pub(crate) already_absent_root_count: usize,
    pub(crate) validator_output_path: String,
    pub(crate) validator_violation_count: usize,
    pub(crate) validator_ok: bool,
    pub(crate) validator_readiness_snapshot_count: usize,
    pub(crate) validator_readiness_json_snapshot_count: usize,
    pub(crate) validator_readiness_tsv_snapshot_count: usize,
    pub(crate) ok: bool,
    pub(crate) deleted_roots: Vec<DisposableRootDeletionStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LegacyFixtureWrapperStatus {
    pub(crate) wrapper_path: String,
    pub(crate) expected_target: String,
    pub(crate) actual_target: Option<String>,
    pub(crate) exists: bool,
    pub(crate) is_symlink: bool,
    pub(crate) root_tests_readme_path: String,
    pub(crate) root_tests_readme_exists: bool,
    pub(crate) root_tests_readme_tracked_by_git: bool,
    pub(crate) root_tests_regular_file_count: usize,
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

pub(crate) fn run_disposable_root_cleanup_proof_command(
    cwd: &Path,
    args: &parse::BenchPathsCleanupProofArgs,
) -> Result<()> {
    let report = prove_disposable_root_cleanup(
        cwd,
        PathBuf::from(DEFAULT_DISPOSABLE_ROOT_CLEANUP_PROOF_PATH),
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
    let legacy_fixture_wrapper = legacy_fixture_wrapper_status(repo_root)?;
    let readiness_snapshots =
        collect_readiness_snapshots(&repo_root.join("benchmarks/readiness"), repo_root)?;
    let readiness_snapshot_count = readiness_snapshots.len();
    let readiness_json_snapshot_count =
        readiness_snapshots.iter().filter(|path| path.ends_with(".json")).count();
    let readiness_tsv_snapshot_count =
        readiness_snapshots.iter().filter(|path| path.ends_with(".tsv")).count();
    let violations = collect_path_violations(&roots, &legacy_fixture_wrapper, &readiness_snapshots);
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
        readiness_snapshot_count,
        readiness_json_snapshot_count,
        readiness_tsv_snapshot_count,
        root_tests_regular_file_count: legacy_fixture_wrapper.root_tests_regular_file_count,
        root_tests_readme_tracked_by_git: legacy_fixture_wrapper.root_tests_readme_tracked_by_git,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        legacy_fixture_wrapper,
        readiness_snapshots,
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

pub(crate) fn prove_disposable_root_cleanup(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<DisposableRootCleanupProofReport> {
    let deleted_roots = DISPOSABLE_ROOTS
        .iter()
        .map(|relative_path| delete_disposable_root(repo_root, relative_path))
        .collect::<Result<Vec<_>>>()?;
    let deleted_root_count = deleted_roots.iter().filter(|root| root.existed_before).count();
    let already_absent_root_count =
        deleted_roots.iter().filter(|root| !root.existed_before).count();
    let validator_report = validate_benchmark_paths(
        repo_root,
        PathBuf::from(DEFAULT_BENCHMARK_PATHS_VALIDATE_PATH),
        false,
    )?;
    let absolute_output_path = repo_root.join(&output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let ok = validator_report.ok && deleted_roots.iter().all(|root| !root.exists_after);
    let report = DisposableRootCleanupProofReport {
        schema_version: DISPOSABLE_ROOT_CLEANUP_PROOF_SCHEMA_VERSION,
        output_path: output_path.display().to_string(),
        repo_root: repo_root.display().to_string(),
        deleted_root_count,
        already_absent_root_count,
        validator_output_path: validator_report.output_path.clone(),
        validator_violation_count: validator_report.violation_count,
        validator_ok: validator_report.ok,
        validator_readiness_snapshot_count: validator_report.readiness_snapshot_count,
        validator_readiness_json_snapshot_count: validator_report.readiness_json_snapshot_count,
        validator_readiness_tsv_snapshot_count: validator_report.readiness_tsv_snapshot_count,
        ok,
        deleted_roots,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;
    if !report.ok {
        let summary = report
            .deleted_roots
            .iter()
            .filter(|root| root.exists_after)
            .map(|root| root.relative_path.as_str())
            .collect::<Vec<_>>();
        if !summary.is_empty() {
            bail!(
                "disposable root cleanup proof failed because disposable roots remain present: {}",
                summary.join(", ")
            );
        }
        bail!(
            "disposable root cleanup proof failed because benchmark path validation reported {} violation(s)",
            report.validator_violation_count
        );
    }
    validate_benchmark_paths(
        repo_root,
        PathBuf::from(DEFAULT_BENCHMARK_PATHS_VALIDATE_PATH),
        true,
    )?;
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

fn delete_disposable_root(
    repo_root: &Path,
    relative_path: &str,
) -> Result<DisposableRootDeletionStatus> {
    let absolute_path = repo_root.join(relative_path);
    let existed_before = absolute_path.exists();
    let removal_action = if !existed_before {
        "already_absent".to_string()
    } else {
        let metadata = std::fs::symlink_metadata(&absolute_path)
            .with_context(|| format!("read metadata for {}", absolute_path.display()))?;
        let file_type = metadata.file_type();
        if file_type.is_dir() && !file_type.is_symlink() {
            remove_directory_tree(&absolute_path)?;
        } else {
            std::fs::remove_file(&absolute_path)
                .with_context(|| format!("remove file {}", absolute_path.display()))?;
        }
        "deleted".to_string()
    };
    Ok(DisposableRootDeletionStatus {
        relative_path: relative_path.to_string(),
        existed_before,
        exists_after: absolute_path.exists(),
        removal_action,
    })
}

fn remove_directory_tree(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let status = Command::new("rm")
            .arg("-rf")
            .arg(path)
            .status()
            .with_context(|| format!("remove directory {}", path.display()))?;
        if !status.success() {
            bail!("remove directory {} returned {}", path.display(), status);
        }
        return Ok(());
    }
    #[cfg(not(unix))]
    {
        std::fs::remove_dir_all(path)
            .with_context(|| format!("remove directory {}", path.display()))
    }
}

fn legacy_fixture_wrapper_status(repo_root: &Path) -> Result<LegacyFixtureWrapperStatus> {
    let wrapper_path = repo_root.join(LEGACY_FIXTURE_WRAPPER_PATH);
    let metadata = std::fs::symlink_metadata(&wrapper_path).ok();
    let is_symlink = metadata
        .as_ref()
        .map(std::fs::Metadata::file_type)
        .is_some_and(|file_type| file_type.is_symlink());
    let actual_target = if is_symlink {
        Some(
            std::fs::read_link(&wrapper_path)
                .with_context(|| format!("read symlink {}", wrapper_path.display()))?
                .display()
                .to_string(),
        )
    } else {
        None
    };
    let root_tests_readme_path = repo_root.join(ROOT_TESTS_README_PATH);
    let root_tests_regular_file_count =
        count_regular_files_without_following_symlinks(&repo_root.join("tests"))?;
    Ok(LegacyFixtureWrapperStatus {
        wrapper_path: LEGACY_FIXTURE_WRAPPER_PATH.to_string(),
        expected_target: LEGACY_FIXTURE_WRAPPER_TARGET.to_string(),
        actual_target,
        exists: metadata.is_some(),
        is_symlink,
        root_tests_readme_path: ROOT_TESTS_README_PATH.to_string(),
        root_tests_readme_exists: root_tests_readme_path.is_file(),
        root_tests_readme_tracked_by_git: git_check_tracked(repo_root, ROOT_TESTS_README_PATH)?,
        root_tests_regular_file_count,
    })
}

fn collect_path_violations(
    roots: &[BenchmarkRootStatus],
    legacy_fixture_wrapper: &LegacyFixtureWrapperStatus,
    readiness_snapshots: &[String],
) -> Vec<BenchmarkPathViolation> {
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
    if !legacy_fixture_wrapper.exists {
        violations.push(BenchmarkPathViolation {
            relative_path: legacy_fixture_wrapper.wrapper_path.clone(),
            violation_type: "missing_legacy_fixture_wrapper".to_string(),
            detail: "root tests fixture compatibility path is absent".to_string(),
        });
    } else if !legacy_fixture_wrapper.is_symlink {
        violations.push(BenchmarkPathViolation {
            relative_path: legacy_fixture_wrapper.wrapper_path.clone(),
            violation_type: "legacy_fixture_wrapper_not_symlink".to_string(),
            detail: "root tests fixture compatibility path must be a symlink".to_string(),
        });
    } else if legacy_fixture_wrapper.actual_target.as_deref()
        != Some(legacy_fixture_wrapper.expected_target.as_str())
    {
        violations.push(BenchmarkPathViolation {
            relative_path: legacy_fixture_wrapper.wrapper_path.clone(),
            violation_type: "legacy_fixture_wrapper_target_drift".to_string(),
            detail: format!(
                "expected target {} but found {}",
                legacy_fixture_wrapper.expected_target,
                legacy_fixture_wrapper.actual_target.as_deref().unwrap_or("<missing>")
            ),
        });
    }
    if !legacy_fixture_wrapper.root_tests_readme_exists {
        violations.push(BenchmarkPathViolation {
            relative_path: legacy_fixture_wrapper.root_tests_readme_path.clone(),
            violation_type: "missing_root_tests_readme".to_string(),
            detail: "root tests README is absent".to_string(),
        });
    }
    if !legacy_fixture_wrapper.root_tests_readme_tracked_by_git {
        violations.push(BenchmarkPathViolation {
            relative_path: legacy_fixture_wrapper.root_tests_readme_path.clone(),
            violation_type: "untracked_root_tests_readme".to_string(),
            detail: "git does not track the root tests README".to_string(),
        });
    }
    if legacy_fixture_wrapper.root_tests_regular_file_count > 1 {
        violations.push(BenchmarkPathViolation {
            relative_path: "tests".to_string(),
            violation_type: "unexpected_root_tests_files".to_string(),
            detail: format!(
                "root tests stores {} regular files; only the pointer README is allowed",
                legacy_fixture_wrapper.root_tests_regular_file_count
            ),
        });
    }
    if !readiness_snapshots.iter().any(|path| path.ends_with(".json")) {
        violations.push(BenchmarkPathViolation {
            relative_path: "benchmarks/readiness".to_string(),
            violation_type: "missing_readiness_json_snapshot".to_string(),
            detail: "tracked readiness proof must include at least one .json snapshot".to_string(),
        });
    }
    if !readiness_snapshots.iter().any(|path| path.ends_with(".tsv")) {
        violations.push(BenchmarkPathViolation {
            relative_path: "benchmarks/readiness".to_string(),
            violation_type: "missing_readiness_tsv_snapshot".to_string(),
            detail: "tracked readiness proof must include at least one .tsv snapshot".to_string(),
        });
    }
    violations
}

fn collect_readiness_snapshots(root: &Path, repo_root: &Path) -> Result<Vec<String>> {
    let mut snapshots = Vec::new();
    collect_readiness_snapshots_recursive(root, repo_root, &mut snapshots)?;
    snapshots.sort();
    Ok(snapshots)
}

fn collect_readiness_snapshots_recursive(
    path: &Path,
    repo_root: &Path,
    snapshots: &mut Vec<String>,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in
        std::fs::read_dir(path).with_context(|| format!("read directory {}", path.display()))?
    {
        let entry = entry.with_context(|| format!("read entry in {}", path.display()))?;
        let entry_path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("read file type for {}", entry_path.display()))?;
        if file_type.is_dir() {
            collect_readiness_snapshots_recursive(&entry_path, repo_root, snapshots)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let extension = entry_path.extension().and_then(|value| value.to_str());
        if !matches!(extension, Some("json" | "tsv")) {
            continue;
        }
        let relative_path = entry_path
            .strip_prefix(repo_root)
            .with_context(|| format!("strip repo root from {}", entry_path.display()))?;
        let relative_path = relative_path.display().to_string();
        if git_check_tracked(repo_root, &relative_path)? {
            snapshots.push(relative_path);
        }
    }
    Ok(())
}

fn count_regular_files_without_following_symlinks(path: &Path) -> Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    let mut count = 0usize;
    for entry in
        std::fs::read_dir(path).with_context(|| format!("read directory {}", path.display()))?
    {
        let entry = entry.with_context(|| format!("read entry in {}", path.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("read file type for {}", entry.path().display()))?;
        if file_type.is_file() {
            count += 1;
        } else if file_type.is_dir() {
            count += count_regular_files_without_following_symlinks(&entry.path())?;
        }
    }
    Ok(count)
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
        LegacyFixtureWrapperStatus, DEFAULT_BENCHMARK_PATHS_VALIDATE_PATH,
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
            .args(["add", "benchmarks", "tests", ".gitignore"])
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
        write_text(&root.join("tests/README.md"), "# Root Tests\n");
        std::fs::create_dir_all(root.join("tests")).expect("create tests root");
        #[cfg(unix)]
        std::os::unix::fs::symlink("../benchmarks/tests/fixtures", root.join("tests/fixtures"))
            .expect("symlink tests fixtures");
    }

    #[test]
    fn benchmark_path_validation_accepts_tracked_root_markers() {
        let temp = tempfile::tempdir().expect("tempdir");
        init_repo(temp.path());
        write_benchmark_root(temp.path());
        write_text(
            &temp.path().join("benchmarks/readiness/all-domain-schema-validation.json"),
            "{\n  \"ok\": true\n}\n",
        );
        write_text(
            &temp.path().join("benchmarks/readiness/all-domain-stage-tool-table.tsv"),
            "stage_id\ttool_id\nvcf.call\tbcftools\n",
        );
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
        assert_eq!(report.readiness_snapshot_count, 2);
        assert_eq!(report.readiness_json_snapshot_count, 1);
        assert_eq!(report.readiness_tsv_snapshot_count, 1);
        assert_eq!(report.root_tests_regular_file_count, 1);
        assert!(report.root_tests_readme_tracked_by_git);
        assert!(report.legacy_fixture_wrapper.exists);
        assert!(report.legacy_fixture_wrapper.is_symlink);
        assert_eq!(
            report.legacy_fixture_wrapper.actual_target.as_deref(),
            Some("../benchmarks/tests/fixtures")
        );
        assert_eq!(
            report.readiness_snapshots,
            vec![
                "benchmarks/readiness/all-domain-schema-validation.json".to_string(),
                "benchmarks/readiness/all-domain-stage-tool-table.tsv".to_string(),
            ]
        );
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
        let legacy_fixture_wrapper = LegacyFixtureWrapperStatus {
            wrapper_path: "tests/fixtures".to_string(),
            expected_target: "../benchmarks/tests/fixtures".to_string(),
            actual_target: Some("../benchmarks/tests/fixtures".to_string()),
            exists: true,
            is_symlink: true,
            root_tests_readme_path: "tests/README.md".to_string(),
            root_tests_readme_exists: true,
            root_tests_readme_tracked_by_git: true,
            root_tests_regular_file_count: 1,
        };
        let violations = collect_path_violations(&roots, &legacy_fixture_wrapper, &[]);
        assert_eq!(violations.len(), 4);
        assert_eq!(violations[0].violation_type, "missing_marker");
        assert_eq!(violations[1].violation_type, "untracked_marker");
        assert_eq!(violations[2].violation_type, "missing_readiness_json_snapshot");
        assert_eq!(violations[3].violation_type, "missing_readiness_tsv_snapshot");
    }

    #[test]
    fn benchmark_path_violation_collection_detects_directory_backed_legacy_fixture_root() {
        let roots = vec![];
        let legacy_fixture_wrapper = LegacyFixtureWrapperStatus {
            wrapper_path: "tests/fixtures".to_string(),
            expected_target: "../benchmarks/tests/fixtures".to_string(),
            actual_target: None,
            exists: true,
            is_symlink: false,
            root_tests_readme_path: "tests/README.md".to_string(),
            root_tests_readme_exists: true,
            root_tests_readme_tracked_by_git: true,
            root_tests_regular_file_count: 3,
        };
        let violations = collect_path_violations(
            &roots,
            &legacy_fixture_wrapper,
            &[
                "benchmarks/readiness/all-domain-schema-validation.json".to_string(),
                "benchmarks/readiness/all-domain-stage-tool-table.tsv".to_string(),
            ],
        );
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].violation_type, "legacy_fixture_wrapper_not_symlink");
        assert_eq!(violations[1].violation_type, "unexpected_root_tests_files");
    }
}
