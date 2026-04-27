#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__configs_layout_policy__configs_root_contains_only_index_and_directories() {
    let root = workspace_root();
    let configs = root.join("configs");
    let mut offenders = Vec::new();

    for entry in std::fs::read_dir(&configs).expect("read configs/") {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let rel = path.strip_prefix(&root).unwrap_or(&path).display().to_string();
        if rel != "configs/index.md" && rel != "configs/OWNERS.toml" {
            offenders.push(rel);
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "configs root must not contain files other than configs/index.md:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__configs_layout_policy__rust_src_uses_configs_path_helper() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let rel = path.strip_prefix(&root).unwrap_or(path).display().to_string();
        if rel == "crates/bijux-dna-infra/src/paths/config.rs" {
            continue;
        }
        if !rel.contains("/src/") {
            continue;
        }

        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if raw.contains("join(\"configs\")")
            || raw.contains("Path::new(\"configs\")")
            || raw.contains("cwd.join(\"configs/")
        {
            offenders.push(rel);
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Rust src must use bijux_dna_infra::configs_file/configs_dir helpers:\n{}",
        offenders.join("\n")
    );
}
