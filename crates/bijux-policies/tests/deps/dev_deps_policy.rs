#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn crate_dirs() -> Vec<PathBuf> {
    let root = workspace_root().join("crates");
    let mut crates = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return crates;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.join("Cargo.toml").exists() {
            crates.push(path);
        }
    }
    crates.sort();
    crates
}

fn parse_dev_dependencies(manifest: &Path) -> Vec<String> {
    let content = std::fs::read_to_string(manifest).expect("read Cargo.toml");
    let mut deps = Vec::new();
    let mut in_dev = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_dev = line == "[dev-dependencies]";
            continue;
        }
        if !in_dev || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = line.split_once('=') {
            let name = name.trim().trim_matches('"').to_string();
            if !name.is_empty() {
                deps.push(name);
            }
        }
    }
    deps
}

#[test]
fn policy__deps__dev_deps_policy__dev_dependencies_are_allowlisted() {
    let allowlist: BTreeSet<&str> = BTreeSet::from([
        "anyhow.workspace",
        "assert_cmd",
        "bijux-benchmark",
        "bijux-domain-fastq",
        "bijux-infra",
        "bijux-pipelines",
        "bijux-planner-bam",
        "bijux-planner-fastq",
        "bijux-policies",
        "bijux-policies.workspace",
        "bijux-testkit",
        "cargo_metadata",
        "insta",
        "insta.workspace",
        "predicates",
        "serde_json.workspace",
        "sha2.workspace",
        "tempfile",
        "tempfile.workspace",
        "uuid",
        "walkdir",
        "walkdir.workspace",
    ]);
    let mut offenders = Vec::new();
    for crate_dir in crate_dirs() {
        let manifest = crate_dir.join("Cargo.toml");
        let deps = parse_dev_dependencies(&manifest);
        for dep in deps {
            if !allowlist.contains(dep.as_str()) {
                offenders.push(format!(
                    "{} dev-dep not allowlisted: {}",
                    manifest.display(),
                    dep
                ));
            }
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "dev-dependencies must be allowlisted:\n{}",
        offenders.join("\n")
    );
}
