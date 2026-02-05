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
