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
fn policy__boundaries__dependency_boundaries__stages_do_not_depend_on_environment() {
    let root = repo_root();
    let manifests = [
        root.join("crates/bijux-dna-stages-fastq/Cargo.toml"),
        root.join("crates/bijux-dna-stages-bam/Cargo.toml"),
    ];
    let mut offenders = Vec::new();
    for manifest in manifests {
        let deps = parse_dependency_names(&manifest);
        if deps.iter().any(|dep| dep == "bijux-dna-environment") {
            offenders.push(format!("{} depends on bijux-dna-environment", manifest.display()));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "stages crates must not depend on bijux-dna-environment:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__stages_and_planners_do_not_depend_on_runner_or_engine(
) {
    let root = repo_root();
    let manifests = [
        root.join("crates/bijux-dna-stages-fastq/Cargo.toml"),
        root.join("crates/bijux-dna-stages-bam/Cargo.toml"),
        root.join("crates/bijux-dna-planner-fastq/Cargo.toml"),
        root.join("crates/bijux-dna-planner-bam/Cargo.toml"),
    ];
    let denylist = ["bijux-dna-runner", "bijux-dna-engine"];
    let mut offenders = Vec::new();
    for manifest in manifests {
        let deps = parse_dependency_names(&manifest);
        for denied in denylist {
            if deps.iter().any(|dep| dep == denied) {
                offenders.push(format!("{} depends on {}", manifest.display(), denied));
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "stages/planners must not depend on runner or engine:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__analyze_and_benchmark_do_not_depend_on_engine() {
    let root = repo_root();
    let manifests = [
        root.join("crates/bijux-dna-analyze/Cargo.toml"),
        root.join("crates/bijux-dna-bench/Cargo.toml"),
        root.join("crates/bijux-dna-bench-model/Cargo.toml"),
    ];
    let mut offenders = Vec::new();
    for manifest in manifests {
        let deps = parse_dependency_names(&manifest);
        if deps.iter().any(|dep| dep == "bijux-dna-engine") {
            offenders.push(format!("{} depends on bijux-dna-engine", manifest.display()));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "analyze/benchmark crates must not depend on bijux-dna-engine:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__engine_has_no_domain_or_stage_dependencies() {
    let root = repo_root();
    let manifest = root.join("crates/bijux-dna-engine/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let denylist = [
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-pipelines",
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
        "bijux-dna-engine must not depend on domain/stages/pipelines:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__runner_has_no_domain_or_stage_dependencies() {
    let root = repo_root();
    let manifest = root.join("crates/bijux-dna-runner/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let denylist = [
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-pipelines",
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
        "bijux-dna-runner must not depend on domain/stages/pipelines:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__runner_does_not_depend_on_engine() {
    let root = repo_root();
    let manifest = root.join("crates/bijux-dna-runner/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    bijux_dna_policies::policy_assert!(
        deps.iter().all(|dep| dep != "bijux-dna-engine"),
        "bijux-dna-runner must not depend on bijux-dna-engine"
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__infra_has_no_domain_or_stage_dependencies() {
    let root = repo_root();
    let manifest = root.join("crates/bijux-dna-infra/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let denylist = [
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-bam",
        "bijux-dna-pipelines",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-infra must not depend on domain/stages/planners:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__pipelines_do_not_depend_on_stages_or_execution() {
    let root = repo_root();
    let manifest = root.join("crates/bijux-dna-pipelines/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let denylist = [
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-engine",
        "bijux-dna-runner",
        "bijux-dna-environment",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-pipelines must not depend on stages/engine/runner:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__environment_has_no_engine_or_runner_dependencies() {
    let root = repo_root();
    let manifest = root.join("crates/bijux-dna-environment/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let denylist = ["bijux-dna-engine", "bijux-dna-runner"];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-environment must not depend on engine/runner:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__production_crates_do_not_depend_on_environment_qa() {
    let root = repo_root();
    let crates_dir = root.join("crates");
    let crate_dirs = std::fs::read_dir(&crates_dir)
        .unwrap_or_else(|err| panic!("read {}: {err}", crates_dir.display()));
    let mut offenders = Vec::new();
    for entry in crate_dirs.flatten() {
        let path = entry.path().join("Cargo.toml");
        if !path.exists() {
            continue;
        }
        if path.to_string_lossy().contains("bijux-dna-environment-qa") {
            continue;
        }
        let deps = parse_dependency_names(&path);
        if deps.iter().any(|dep| dep == "bijux-dna-environment-qa") {
            offenders.push(format!("{} depends on bijux-dna-environment-qa", path.display()));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "production crates must not depend on bijux-dna-environment-qa:\n{}",
        offenders.join("\n")
    );
}
