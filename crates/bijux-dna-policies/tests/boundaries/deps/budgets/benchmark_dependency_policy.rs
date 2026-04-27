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
fn policy__boundaries__benchmark_dependency_policy__benchmark_has_no_planner_or_stage_dependencies()
{
    let root = repo_root();
    let manifest = root.join("crates/bijux-dna-bench/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let denylist = [
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-bam",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-bench must not depend on stages/planners:\n{}",
        offenders.join("\n")
    );
}
