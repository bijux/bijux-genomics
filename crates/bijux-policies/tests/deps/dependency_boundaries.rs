use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn parse_dependency_names(manifest: &Path) -> Vec<String> {
    let content = std::fs::read_to_string(manifest).expect("read Cargo.toml");
    let mut deps = Vec::new();
    let mut in_deps = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = matches!(
                line,
                "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
            );
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
fn stages_do_not_depend_on_environment() {
    let root = workspace_root();
    let manifests = [
        root.join("crates/bijux-stages-fastq/Cargo.toml"),
        root.join("crates/bijux-stages-bam/Cargo.toml"),
    ];
    let mut offenders = Vec::new();
    for manifest in manifests {
        let deps = parse_dependency_names(&manifest);
        if deps.iter().any(|dep| dep == "bijux-environment") {
            offenders.push(format!(
                "{} depends on bijux-environment",
                manifest.display()
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "stages crates must not depend on bijux-environment:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn analyze_and_benchmark_do_not_depend_on_engine() {
    let root = workspace_root();
    let manifests = [
        root.join("crates/bijux-analyze/Cargo.toml"),
        root.join("crates/bijux-benchmark/Cargo.toml"),
        root.join("crates/bijux-benchmark-model/Cargo.toml"),
    ];
    let mut offenders = Vec::new();
    for manifest in manifests {
        let deps = parse_dependency_names(&manifest);
        if deps.iter().any(|dep| dep == "bijux-engine") {
            offenders.push(format!("{} depends on bijux-engine", manifest.display()));
        }
    }
    assert!(
        offenders.is_empty(),
        "analyze/benchmark crates must not depend on bijux-engine:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn engine_has_no_domain_or_stage_dependencies() {
    let root = workspace_root();
    let manifest = root.join("crates/bijux-engine/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let denylist = [
        "bijux-domain-fastq",
        "bijux-domain-bam",
        "bijux-stages-fastq",
        "bijux-stages-bam",
        "bijux-pipelines",
        "bijux-planner-fastq",
        "bijux-planner-bam",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    assert!(
        offenders.is_empty(),
        "bijux-engine must not depend on domain/stages/pipelines:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn runner_has_no_domain_or_stage_dependencies() {
    let root = workspace_root();
    let manifest = root.join("crates/bijux-runner/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let denylist = [
        "bijux-domain-fastq",
        "bijux-domain-bam",
        "bijux-stages-fastq",
        "bijux-stages-bam",
        "bijux-pipelines",
        "bijux-planner-fastq",
        "bijux-planner-bam",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    assert!(
        offenders.is_empty(),
        "bijux-runner must not depend on domain/stages/pipelines:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn runner_does_not_depend_on_engine() {
    let root = workspace_root();
    let manifest = root.join("crates/bijux-runner/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    assert!(
        deps.iter().all(|dep| dep != "bijux-engine"),
        "bijux-runner must not depend on bijux-engine"
    );
}

#[test]
fn infra_has_no_domain_or_stage_dependencies() {
    let root = workspace_root();
    let manifest = root.join("crates/bijux-infra/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let denylist = [
        "bijux-domain-fastq",
        "bijux-domain-bam",
        "bijux-stages-fastq",
        "bijux-stages-bam",
        "bijux-planner-fastq",
        "bijux-planner-bam",
        "bijux-pipelines",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    assert!(
        offenders.is_empty(),
        "bijux-infra must not depend on domain/stages/planners:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn pipelines_do_not_depend_on_stages_or_execution() {
    let root = workspace_root();
    let manifest = root.join("crates/bijux-pipelines/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let denylist = [
        "bijux-stages-fastq",
        "bijux-stages-bam",
        "bijux-engine",
        "bijux-runner",
        "bijux-environment",
        "bijux-runtime",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    assert!(
        offenders.is_empty(),
        "bijux-pipelines must not depend on stages/engine/runner:\n{}",
        offenders.join("\n")
    );
}
