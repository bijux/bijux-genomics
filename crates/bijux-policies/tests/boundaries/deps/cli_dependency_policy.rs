#![allow(non_snake_case)]
#![allow(non_snake_case)]
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
            in_deps = matches!(line, "[dependencies]" | "[build-dependencies]");
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
fn policy__boundaries__cli_dependency_policy__cli_depends_only_on_api_and_cli_support() {
    let root = workspace_root();
    let manifest = root.join("crates/bijux-cli/Cargo.toml");
    let deps = parse_dependency_names(&manifest);
    let allowlist = [
        "bijux-api",
        "clap",
        "tracing",
        "anyhow",
        "serde",
        "serde_json",
    ];
    let offenders: Vec<String> = deps
        .iter()
        .filter(|dep| !allowlist.contains(&dep.as_str()))
        .map(|dep| {
            format!(
                "{} depends on unexpected crate: {}",
                manifest.display(),
                dep
            )
        })
        .collect();

    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-cli must depend only on bijux-api + clap + logging (and minimal support libs).
How to fix: move infra/runtime/runner dependencies behind bijux-api or remove them.
Offenders:\n{}",
        offenders.join("\n")
    );
}
