#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
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
    let content = std::fs::read_to_string(manifest)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest.display()));
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
            let name = name.trim().trim_matches('"');
            let name = name.strip_suffix(".workspace").unwrap_or(name).to_string();
            if !name.is_empty() {
                deps.push(name);
            }
        }
    }
    deps
}

#[test]
fn policy__boundaries__dev_deps_policy__dev_dependencies_are_allowlisted() {
    let allowlist: BTreeSet<&str> = BTreeSet::from([
        "anyhow",
        "anyhow.workspace",
        "assert_cmd",
        "assert_cmd.workspace",
        "bijux-dna-bench",
        "bijux-dna-analyze",
        "bijux-dna-core",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-compiler",
        "bijux-dna-domain-fastq",
        "bijux-dna-infra",
        "bijux-dna-pipelines",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-policies",
        "bijux-dna-policies.workspace",
        "bijux-dna-runtime",
        "bijux-dna-stage-contract",
        "bijux-dna-testkit",
        "bijux-dna-testkit.workspace",
        "cargo_metadata",
        "cargo_metadata.workspace",
        "filetime",
        "filetime.workspace",
        "flate2",
        "flate2.workspace",
        "gag",
        "insta",
        "insta.workspace",
        "predicates",
        "predicates.workspace",
        "regex",
        "regex.workspace",
        "serde_yaml",
        "serde_yaml.workspace",
        "serde_json",
        "serde_json.workspace",
        "sha2",
        "sha2.workspace",
        "tempfile",
        "tempfile.workspace",
        "toml",
        "toml.workspace",
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
                offenders.push(format!("{} dev-dep not allowlisted: {}", manifest.display(), dep));
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "dev-dependencies must be allowlisted:\n{}",
        offenders.join("\n")
    );
}
