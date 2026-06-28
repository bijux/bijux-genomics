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
            in_deps = matches!(line, "[dependencies]" | "[build-dependencies]");
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _rest)) = line.split_once('=') {
            let raw_name = name.trim().trim_matches('"');
            let name = raw_name.strip_suffix(".workspace").unwrap_or(raw_name).to_string();
            if !name.is_empty() {
                deps.push(name);
            }
        }
    }
    deps
}

#[test]
fn policy__boundaries__cli_dependency_policy__cli_depends_only_on_documented_adapter_surface() {
    let root = repo_root();
    let manifest = root.join("crates/bijux-dna/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let allowlist = [
        "bijux-dna-analyze",
        "bijux-dna-api",
        "bijux-dna-core",
        "bijux-dna-db-ref",
        "bijux-dna-domain-compiler",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-vcf",
        "bijux-dna-db-ena",
        "bijux-dna-infra",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-vcf",
        "bijux-dna-stage-contract",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-vcf",
        "clap",
        "tracing",
        "anyhow",
        "serde",
        "serde_json",
        "serde_yaml",
        "regex",
        "toml",
        "sha2",
        "tar",
        "flate2",
        "reqwest",
        "schemars",
        "noodles-bam",
        "noodles-sam",
        "zip",
    ];
    let offenders: Vec<String> = deps
        .iter()
        .filter(|dep| !allowlist.contains(&dep.as_str()))
        .map(|dep| format!("{} depends on unexpected crate: {}", manifest.display(), dep))
        .collect();

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna must depend only on its documented CLI adapter surface.
How to fix: remove the dependency or update the documented CLI boundary first, then align this policy.
Offenders:\n{}",
        offenders.join("\n")
    );
}
