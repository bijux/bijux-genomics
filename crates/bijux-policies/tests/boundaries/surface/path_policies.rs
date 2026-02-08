#![allow(non_snake_case)]
#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn is_allowed_writer_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/crates/bijux-runtime/")
        || path_str.contains("/crates/bijux-engine/")
        || path_str.contains("/crates/bijux-cli/src/commands/policies.rs")
}

fn is_path_policies_test(path: &Path) -> bool {
    path.to_string_lossy()
        .ends_with("/crates/bijux-policies/tests/boundaries/surface/path_policies.rs")
}

#[test]
fn policy__boundaries__path_policies__src_bin_requires_bin_targets() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir())
    {
        if entry.file_name() != "src" {
            continue;
        }
        let crate_root = entry.path().parent().unwrap();
        let src_bin = entry.path().join("bin");
        if !src_bin.exists() {
            continue;
        }
        let has_bins = std::fs::read_dir(&src_bin)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .any(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
            })
            .unwrap_or(false);
        if !has_bins {
            offenders.push(crate_root.display().to_string());
        }
    }

    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "src/bin must contain at least one .rs binary source:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__path_policies__src_does_not_contain_test_paths() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() && !entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        if !path.to_string_lossy().contains("/src/") {
            continue;
        }
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
        if name.contains("test") {
            offenders.push(path.display().to_string());
        }
    }

    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "src paths must not include *test* names:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__path_policies__run_artifacts_paths_use_runtime_helpers() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let targets = [
        root.join("crates/bijux-api/src"),
        root.join("crates/bijux-cli/src"),
        root.join("crates/bijux-stages-fastq/src"),
        root.join("crates/bijux-stages-bam/src"),
    ];
    for target in targets {
        for entry in WalkDir::new(target)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let content = std::fs::read_to_string(entry.path()).expect("read source");
            if content.contains("\"run_artifacts\"") {
                offenders.push(entry.path().display().to_string());
            }
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "run_artifacts paths must use bijux_runtime helpers, not string joins:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__path_policies__write_locations_are_confined_to_runtime_and_engine() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let patterns = ["std::fs::OpenOptions", "std::fs::write", "File::create("];

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if entry.path().to_string_lossy().contains("/tests/") {
            continue;
        }
        if is_allowed_writer_path(entry.path()) {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        if patterns.iter().any(|pattern| content.contains(pattern)) {
            offenders.push(entry.path().display().to_string());
        }
    }

    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "direct filesystem writes must be confined to bijux-runtime or bijux-engine:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__path_policies__crates_do_not_reference_removed_fastq_test_paths() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = ["tests/data/fastq/"];
    let exts = ["rs", "toml", "md"];

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => ext,
            None => continue,
        };
        if !exts.contains(&ext) {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        if is_path_policies_test(path) {
            continue;
        }
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(path.display().to_string());
        }
    }

    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "crates must not reference tests/data/fastq paths:\n{}",
        offenders.join("\n")
    );
}
