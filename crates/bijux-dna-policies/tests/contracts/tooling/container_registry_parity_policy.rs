#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;
use walkdir::WalkDir;

fn load_registry_tool_ids_for_runtime(runtime: &str) -> BTreeSet<String> {
    let root = support::workspace_root();
    let mut ids = BTreeSet::new();
    for file in [
        "configs/tool_registry.toml",
        "configs/tool_registry_experimental.toml",
        "configs/tool_registry_vcf.toml",
    ] {
        let path = root.join(file);
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        let parsed: toml::Value = raw
            .parse()
            .unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));
        let Some(rows) = parsed.get("tools").and_then(toml::Value::as_array) else {
            continue;
        };
        for row in rows {
            let id = row
                .get("id")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .trim();
            if id.is_empty() {
                continue;
            }
            let container_enabled = row
                .get("container")
                .and_then(toml::Value::as_bool)
                .unwrap_or(true);
            if !container_enabled {
                continue;
            }
            let is_planned = row
                .get("version")
                .and_then(toml::Value::as_str)
                .is_some_and(|v| v == "planned");
            if is_planned {
                continue;
            }
            let runtimes = row
                .get("runtimes")
                .and_then(toml::Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(toml::Value::as_str)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if runtimes.contains(&runtime) {
                ids.insert(id.to_string());
            }
        }
    }
    ids
}

fn docker_defs() -> BTreeSet<String> {
    let root = support::workspace_root();
    let mut defs = BTreeSet::new();
    for entry in WalkDir::new(root.join("containers/docker"))
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if let Some(id) = name.strip_prefix("Dockerfile.") {
            defs.insert(id.to_string());
        }
    }
    defs
}

fn apptainer_defs() -> BTreeSet<String> {
    let root = support::workspace_root();
    let mut defs = BTreeSet::new();
    for entry in WalkDir::new(root.join("containers/apptainer"))
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|v| v.to_str()) != Some("def") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|v| v.to_str()) {
            defs.insert(stem.to_string());
        }
    }
    defs
}

#[test]
fn policy__contracts__container_registry_parity_policy__registry_runtime_tools_have_container_defs()
{
    let registry_docker = load_registry_tool_ids_for_runtime("docker");
    let registry_apptainer = load_registry_tool_ids_for_runtime("apptainer");
    let defs_docker = docker_defs();
    let defs_apptainer = apptainer_defs();

    let mut missing = Vec::new();
    for tool in registry_docker.difference(&defs_docker) {
        missing.push(format!(
            "missing docker container def for registry tool: {tool}"
        ));
    }
    for tool in registry_apptainer.difference(&defs_apptainer) {
        missing.push(format!(
            "missing apptainer container def for registry tool: {tool}"
        ));
    }

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "registry -> container parity failures:\n{}",
        missing.join("\n")
    );
}

#[test]
fn policy__contracts__container_registry_parity_policy__container_defs_are_registered_tools() {
    let registry_docker = load_registry_tool_ids_for_runtime("docker");
    let registry_apptainer = load_registry_tool_ids_for_runtime("apptainer");
    let defs_docker = docker_defs();
    let defs_apptainer = apptainer_defs();

    let mut orphan = Vec::new();
    for tool in defs_docker.difference(&registry_docker) {
        orphan.push(format!(
            "orphan docker container def not in registry: {tool}"
        ));
    }
    for tool in defs_apptainer.difference(&registry_apptainer) {
        orphan.push(format!(
            "orphan apptainer container def not in registry: {tool}"
        ));
    }

    bijux_dna_policies::policy_assert!(
        orphan.is_empty(),
        "container -> registry parity failures:\n{}",
        orphan.join("\n")
    );
}
