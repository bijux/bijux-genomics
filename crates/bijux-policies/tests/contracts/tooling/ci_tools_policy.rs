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

#[test]
fn policy__tooling__ci_tools_policy__workflows_use_make_only() {
    let root = workspace_root();
    let workflows_dir = root.join(".github").join("workflows");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(workflows_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("yml") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read workflow");
        if content.contains("cargo clippy")
            || content.contains("cargo fmt")
            || content.contains("cargo test")
            || content.contains("cargo nextest")
            || content.contains("cargo make")
        {
            offenders.push(entry.path().display().to_string());
        }
        if !content.contains("make ") {
            offenders.push(entry.path().display().to_string());
        }
    }
    offenders.sort();
    offenders.dedup();
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "CI workflows must use Makefile entrypoints only: {:?}",
        offenders
    );
}

#[test]
fn policy__tooling__ci_tools_policy__serde_yaml_is_scoped() {
    let root = workspace_root();
    let allowed = ["bijux-infra"];
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.file_name() != "Cargo.toml" {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read Cargo.toml");
        if !content.contains("serde_yaml") && !content.contains("serde-yaml") {
            continue;
        }
        let name = entry
            .path()
            .parent()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !allowed.contains(&name) {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "serde_yaml must be scoped to {:?}: {:?}",
        allowed,
        offenders
    );
}
