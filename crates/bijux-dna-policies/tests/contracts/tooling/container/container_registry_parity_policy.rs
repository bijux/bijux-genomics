#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;
use walkdir::WalkDir;

struct RuntimeDefSpec {
    tool_id: String,
    status: String,
    container_ref: String,
    def_path: String,
}

fn registry_files() -> [&'static str; 5] {
    [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
        "configs/ci/registry/tool_registry_container_experimental.toml",
    ]
}

fn load_registry_tool_defs_for_runtime(runtime: &str) -> Vec<RuntimeDefSpec> {
    let root = support::workspace_root();
    let mut defs = Vec::new();
    for file in registry_files() {
        let path = root.join(file);
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        let parsed: toml::Value =
            raw.parse().unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));
        let Some(rows) = parsed.get("tools").and_then(toml::Value::as_array) else {
            continue;
        };
        for row in rows {
            let id = row.get("id").and_then(toml::Value::as_str).unwrap_or_default().trim();
            if id.is_empty() {
                continue;
            }
            let container_enabled =
                row.get("container").and_then(toml::Value::as_bool).unwrap_or(true);
            if !container_enabled {
                continue;
            }
            let runtimes = row
                .get("runtimes")
                .and_then(toml::Value::as_array)
                .map(|arr| arr.iter().filter_map(toml::Value::as_str).collect::<Vec<_>>())
                .unwrap_or_default();
            if !runtimes.contains(&runtime) {
                continue;
            }
            let def_key = if runtime == "docker" { "dockerfile" } else { "apptainer_def" };
            let def_path =
                row.get(def_key).and_then(toml::Value::as_str).unwrap_or_default().trim();
            let status =
                row.get("status").and_then(toml::Value::as_str).unwrap_or_default().to_string();
            let container_ref =
                row.get("container_ref").and_then(toml::Value::as_str).unwrap_or_default();
            defs.push(RuntimeDefSpec {
                tool_id: id.to_string(),
                status,
                container_ref: container_ref.to_string(),
                def_path: def_path.to_string(),
            });
        }
    }
    defs
}

fn docker_defs() -> BTreeSet<String> {
    let root = support::workspace_root();
    let mut defs = BTreeSet::new();
    for entry in
        WalkDir::new(root.join("containers/docker/arm64")).into_iter().filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("def")
            && !entry.file_name().to_string_lossy().starts_with("Dockerfile.")
        {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(&root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        defs.insert(rel);
    }
    defs
}

fn apptainer_defs() -> BTreeSet<String> {
    let root = support::workspace_root();
    let mut defs = BTreeSet::new();
    for entry in
        WalkDir::new(root.join("containers/apptainer/shared")).into_iter().filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|v| v.to_str()) != Some("def") {
            continue;
        }
        let rel = path.strip_prefix(&root).unwrap_or(path).to_string_lossy().replace('\\', "/");
        defs.insert(rel);
    }
    defs
}

#[test]
fn policy__contracts__container_registry_parity_policy__registry_runtime_tools_have_container_defs()
{
    let root = support::workspace_root();
    let registry_docker = load_registry_tool_defs_for_runtime("docker");
    let registry_apptainer = load_registry_tool_defs_for_runtime("apptainer");
    let defs_docker = docker_defs();
    let defs_apptainer = apptainer_defs();

    let mut missing = Vec::new();
    for spec in registry_docker {
        if spec.def_path.is_empty()
            && spec.status == "planned"
            && spec.container_ref.eq_ignore_ascii_case("planned")
        {
            continue;
        }
        if spec.def_path.is_empty() {
            missing.push(format!(
                "missing docker container def path for registry tool: {}",
                spec.tool_id
            ));
            continue;
        }
        let def_path = root.join(&spec.def_path);
        if !def_path.exists() || !defs_docker.contains(&spec.def_path) {
            missing.push(format!(
                "missing docker container def for registry tool: {} -> {}",
                spec.tool_id, spec.def_path
            ));
        }
    }
    for spec in registry_apptainer {
        if spec.def_path.is_empty()
            && spec.status == "planned"
            && spec.container_ref.eq_ignore_ascii_case("planned")
        {
            continue;
        }
        if spec.def_path.is_empty() {
            missing.push(format!(
                "missing apptainer container def path for registry tool: {}",
                spec.tool_id
            ));
            continue;
        }
        let def_path = root.join(&spec.def_path);
        if !def_path.exists() || !defs_apptainer.contains(&spec.def_path) {
            missing.push(format!(
                "missing apptainer container def for registry tool: {} -> {}",
                spec.tool_id, spec.def_path
            ));
        }
    }

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "registry -> container parity drift:\n{}",
        missing.join("\n")
    );
}

#[test]
fn policy__contracts__container_registry_parity_policy__container_defs_are_registered_tools() {
    let registry_docker = load_registry_tool_defs_for_runtime("docker")
        .into_iter()
        .filter_map(|spec| if spec.def_path.is_empty() { None } else { Some(spec.def_path) })
        .collect::<BTreeSet<_>>();
    let registry_apptainer = load_registry_tool_defs_for_runtime("apptainer")
        .into_iter()
        .filter_map(|spec| if spec.def_path.is_empty() { None } else { Some(spec.def_path) })
        .collect::<BTreeSet<_>>();
    let defs_docker = docker_defs();
    let defs_apptainer = apptainer_defs();

    let mut orphan = Vec::new();
    for def in defs_docker.difference(&registry_docker) {
        orphan.push(format!("orphan docker container def not in registry: {def}"));
    }
    for def in defs_apptainer.difference(&registry_apptainer) {
        orphan.push(format!("orphan apptainer container def not in registry: {def}"));
    }

    bijux_dna_policies::policy_assert!(
        orphan.is_empty(),
        "container -> registry parity drift:\n{}",
        orphan.join("\n")
    );
}
