#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn parse_dependency_names(manifest: &Path) -> Vec<String> {
    let content = std::fs::read_to_string(manifest)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest.display()));
    let mut deps = Vec::new();
    let mut in_deps = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps =
                matches!(line, "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]");
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _rest)) = line.split_once('=') {
            let name = name.trim().trim_matches('"').to_string();
            if !name.is_empty() {
                deps.push(name);
            }
        }
    }
    deps
}

#[test]
fn policy__boundaries__domain_dependency_policy__domain_crates_use_only_pure_dependencies() {
    let root = repo_root();
    let denylist = [
        "rusqlite",
        "reqwest",
        "tokio",
        "tracing-subscriber",
        "opendal",
        "bollard",
        "bijux-dna-runtime",
        "bijux-dna-engine",
        "bijux-dna-environment",
    ];
    let domains = [
        root.join("crates/bijux-dna-domain-fastq/Cargo.toml"),
        root.join("crates/bijux-dna-domain-bam/Cargo.toml"),
    ];
    let mut offenders = Vec::new();
    for manifest in domains {
        let deps = parse_dependency_names(&manifest);
        for forbidden in denylist {
            if deps.iter().any(|dep| dep == forbidden) {
                offenders.push(format!("{} depends on {}", manifest.display(), forbidden));
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "domain crates must not depend on external/side-effectful deps:\n{}",
        offenders.join("\n")
    );
}
