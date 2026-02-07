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
fn pipelines_do_not_depend_on_stages_or_planners() {
    let root = workspace_root();
    let manifest = root.join("crates/bijux-pipelines/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let denylist = [
        "bijux-stages-fastq",
        "bijux-stages-bam",
        "bijux-planner-fastq",
        "bijux-planner-bam",
        "bijux-engine",
        "bijux-runner",
        "bijux-runtime",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    assert!(
        offenders.is_empty(),
        "bijux-pipelines must not depend on stages/planners/execution crates:\n{}",
        offenders.join("\n")
    );
}
