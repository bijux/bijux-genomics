#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn rs_test_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let path_s = path.to_string_lossy();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let content = bijux_dna_testkit::read_policy_text(path);
        let is_integration_test = path_s.contains("/tests/");
        let is_unit_test_container = path_s.contains("/src/")
            && (content.contains("#[cfg(test)]") || content.contains("mod tests {"));
        if !is_integration_test && !is_unit_test_container {
            continue;
        }
        files.push(path.to_path_buf());
    }
    files.sort();
    files
}

fn test_scoped_content(path: &Path) -> String {
    let content = bijux_dna_testkit::read_policy_text(path);
    let path_s = path.to_string_lossy();
    if path_s.contains("/tests/") {
        return content;
    }
    if let Some((_, scoped)) = content.split_once("#[cfg(test)]") {
        return scoped.to_string();
    }
    if let Some((_, scoped)) = content.split_once("mod tests {") {
        return scoped.to_string();
    }
    content
}

fn is_policy_file(path: &Path) -> bool {
    path.ends_with(
        "crates/bijux-dna-policies/tests/contracts/tooling/runtime/test_determinism_policy.rs",
    )
}

#[test]
fn policy__contracts__test_determinism_policy__tests_ban_systemtime_now() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for file in rs_test_files(&root) {
        if is_policy_file(&file) {
            continue;
        }
        let content = test_scoped_content(&file);
        if content.contains("SystemTime::now(") || content.contains("std::time::SystemTime::now(") {
            offenders.push(file.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tests must use bijux_dna_testkit::FixedClock instead of SystemTime::now:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__test_determinism_policy__tests_ban_thread_rng() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for file in rs_test_files(&root) {
        if is_policy_file(&file) {
            continue;
        }
        let content = test_scoped_content(&file);
        if content.contains("rand::thread_rng(") || content.contains("thread_rng(") {
            offenders.push(file.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tests must use bijux_dna_testkit::fixed_rng(seed) instead of thread_rng:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__test_determinism_policy__tests_do_not_write_to_artifacts_root() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let write_to_artifacts =
        Regex::new(r#"(write|create_dir|create_dir_all|open)\s*\([^\n]*[\"']artifacts/"#)
            .expect("compile regex");
    let hardcoded_artifacts_root =
        Regex::new(r#"PathBuf::from\(\s*[\"']artifacts(?:/|[\"'])"#).expect("compile regex");

    for file in rs_test_files(&root) {
        let content = test_scoped_content(&file);
        if write_to_artifacts.is_match(&content) || hardcoded_artifacts_root.is_match(&content) {
            offenders.push(file.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tests must write under TEST_TMP_DIR/TestPaths, not hardcoded artifacts/:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__test_determinism_policy__jsonl_appends_use_locked_writer() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let path_s = path.to_string_lossy();
        if path_s.contains("/target/") {
            continue;
        }
        if path_s.ends_with("/bijux-dna-runtime/src/run_layout.rs")
            || path_s.ends_with("/bijux-dna-runtime/src/run_layout/journal.rs")
            || path_s.ends_with("/bijux-dna/src/commands/bench_suite/bench_suite_part1.rs")
            || path_s.ends_with("/bijux-dna-core/tests/contracts/identity/run_index.rs")
        {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        let writes_jsonl = content.contains(".jsonl")
            && (content.contains("OpenOptions::new") || content.contains("append(true)"));
        let uses_locked_helpers = content.contains("append_jsonl_line(")
            || content.contains("append_execution_event(")
            || content.contains("append_run_index_record(");
        if writes_jsonl && !uses_locked_helpers {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "jsonl appends must route through locked writer helpers:\n{}",
        offenders.join("\n")
    );
}
