#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use regex::Regex;
use support::workspace_root;
use walkdir::WalkDir;

fn as_table_array<'a>(value: &'a toml::Value, key: &str) -> Vec<&'a toml::Value> {
    value
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|arr| arr.iter().collect())
        .unwrap_or_default()
}

fn as_str_field<'a>(table: &'a toml::Value, key: &str) -> Option<&'a str> {
    table.get(key).and_then(toml::Value::as_str)
}

fn file_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_string()
}

fn runtimes(table: &toml::Value) -> Vec<String> {
    table
        .get("runtimes")
        .and_then(toml::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn as_bool_field(table: &toml::Value, key: &str, default: bool) -> bool {
    table
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(default)
}

#[test]
fn policy__contracts__tool_registry_completeness__registry_entries_are_machine_checkable() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let legacy_registry = root.join("configs/tools.toml");
    if legacy_registry.exists() {
        offenders.push(format!(
            "configs/tools.toml is deprecated; use generated configs/tool_registry.toml only: {}",
            legacy_registry.display()
        ));
    }
    let registry_path = root.join("configs/tool_registry.toml");
    let experimental_registry_path = root.join("configs/tool_registry_experimental.toml");
    let vcf_registry_path = root.join("configs/tool_registry_vcf.toml");
    let raw = std::fs::read_to_string(&registry_path).expect("read configs/tool_registry.toml");
    let parsed: toml::Value = raw.parse().expect("parse configs/tool_registry.toml");
    let experimental_raw = std::fs::read_to_string(&experimental_registry_path)
        .expect("read configs/tool_registry_experimental.toml");
    let experimental_parsed: toml::Value = experimental_raw
        .parse()
        .expect("parse configs/tool_registry_experimental.toml");
    let vcf_raw =
        std::fs::read_to_string(&vcf_registry_path).expect("read configs/tool_registry_vcf.toml");
    let vcf_parsed: toml::Value = vcf_raw
        .parse()
        .expect("parse configs/tool_registry_vcf.toml");
    let mut tools = as_table_array(&parsed, "tools");
    tools.extend(as_table_array(&experimental_parsed, "tools"));
    tools.extend(as_table_array(&vcf_parsed, "tools"));
    let mut declared_docker_tool_files = std::collections::BTreeSet::new();
    let mut declared_apptainer_tool_files = std::collections::BTreeSet::new();
    let checkout_commit_re =
        Regex::new(r"git checkout [0-9a-f]{40}").expect("compile git checkout regex");

    if tools.is_empty() {
        offenders.push("configs/tool_registry.toml: missing [[tools]] entries".to_string());
    }

    for entry in tools {
        let id = as_str_field(entry, "id").unwrap_or("<missing>");
        for required in [
            "id",
            "version",
            "upstream",
            "pinned_commit",
            "version_cmd",
            "expected_version_regex",
            "healthcheck_cmd",
        ] {
            let value = as_str_field(entry, required).unwrap_or("");
            if value.trim().is_empty() {
                offenders.push(format!("tool={id}: missing required field `{required}`"));
            }
        }

        let container_enabled = as_bool_field(entry, "container", true);
        let is_planned = as_str_field(entry, "version").is_some_and(|version| version == "planned");
        let runtimes = runtimes(entry);
        if runtimes.is_empty() && !is_planned {
            offenders.push(format!("tool={id}: `runtimes` must be non-empty"));
        }
        if container_enabled && runtimes.len() == 1 {
            let reason = as_str_field(entry, "runtime_rationale")
                .unwrap_or("")
                .trim();
            if reason.is_empty() {
                offenders.push(format!(
                    "tool={id}: single-runtime tools must define runtime_rationale"
                ));
            }
        }

        for runtime in &runtimes {
            if !container_enabled || is_planned {
                continue;
            }
            match runtime.as_str() {
                "docker" => {
                    let path = as_str_field(entry, "dockerfile").unwrap_or("");
                    if path.is_empty() {
                        offenders.push(format!("tool={id}: missing dockerfile path"));
                    } else {
                        if !path.starts_with("containers/docker/") {
                            offenders.push(format!(
                                "tool={id}: dockerfile path must be under containers/docker/: {path}"
                            ));
                        }
                        let expected = format!("Dockerfile.{id}");
                        let actual = file_name(path);
                        if actual != expected {
                            offenders.push(format!(
                                "tool={id}: dockerfile must follow Dockerfile.<tool> naming: expected {expected}, got {actual}"
                            ));
                        }
                        declared_docker_tool_files.insert(actual.replace("Dockerfile.", ""));
                        let abs = root.join(path);
                        if !abs.exists() {
                            offenders.push(format!("tool={id}: dockerfile not found at {path}"));
                        } else {
                            let content = std::fs::read_to_string(&abs).unwrap_or_default();
                            if !content.contains("SPDX-License-Identifier: ") {
                                offenders
                                    .push(format!("tool={id}: dockerfile missing SPDX header"));
                            }
                            if !content.contains("org.opencontainers.image.licenses=") {
                                offenders.push(format!(
                                    "tool={id}: dockerfile missing OCI license label"
                                ));
                            }
                            for required in [
                                "org.opencontainers.image.source=",
                                "org.opencontainers.image.revision=",
                                "org.opencontainers.image.created=",
                                "org.opencontainers.image.version=",
                                "org.opencontainers.image.tool=",
                                "org.opencontainers.image.base.name=",
                                "org.opencontainers.image.base.digest=",
                                "ARG TOOL_VERSION",
                            ] {
                                if !content.contains(required) {
                                    offenders.push(format!(
                                        "tool={id}: dockerfile missing reproducibility metadata marker `{required}`"
                                    ));
                                }
                            }
                            if content.contains("git clone")
                                && !checkout_commit_re.is_match(&content)
                            {
                                offenders.push(format!(
                                    "tool={id}: dockerfile uses git clone without immutable commit checkout"
                                ));
                            }
                            if content.contains("container scaffold")
                                || content.contains("executable not yet wired")
                            {
                                offenders.push(format!(
                                    "tool={id}: dockerfile still contains scaffold placeholder text"
                                ));
                            }
                        }
                    }
                }
                "apptainer" => {
                    let path = as_str_field(entry, "apptainer_def").unwrap_or("");
                    if path.is_empty() {
                        offenders.push(format!("tool={id}: missing apptainer_def path"));
                    } else {
                        if !path.starts_with("containers/apptainer/bijux/")
                            && !path.starts_with("containers/apptainer/non-bijux/")
                        {
                            offenders.push(format!(
                                "tool={id}: apptainer def path must be under containers/apptainer/bijux/ or containers/apptainer/non-bijux/: {path}"
                            ));
                        }
                        let expected = format!("{id}.def");
                        let actual = file_name(path);
                        if actual != expected {
                            offenders.push(format!(
                                "tool={id}: apptainer def must follow <tool>.def naming: expected {expected}, got {actual}"
                            ));
                        }
                        declared_apptainer_tool_files.insert(actual.replace(".def", ""));
                        let abs = root.join(path);
                        if !abs.exists() {
                            offenders.push(format!("tool={id}: apptainer def not found at {path}"));
                        } else {
                            let content = std::fs::read_to_string(&abs).unwrap_or_default();
                            if !content.contains("Container definition license: ") {
                                offenders.push(format!(
                                    "tool={id}: apptainer def missing license header"
                                ));
                            }
                            if !content.contains("org.opencontainers.image.licenses ") {
                                offenders.push(format!(
                                    "tool={id}: apptainer def missing OCI license label"
                                ));
                            }
                            for required in [
                                "org.opencontainers.image.source ",
                                "org.opencontainers.image.revision ",
                                "org.opencontainers.image.created ",
                                "org.opencontainers.image.version ",
                                "org.opencontainers.image.tool ",
                            ] {
                                if !content.contains(required) {
                                    offenders.push(format!(
                                        "tool={id}: apptainer def missing reproducibility metadata marker `{required}`"
                                    ));
                                }
                            }
                            if content.contains("git clone")
                                && !checkout_commit_re.is_match(&content)
                            {
                                offenders.push(format!(
                                    "tool={id}: apptainer def uses git clone without immutable commit checkout"
                                ));
                            }
                            if content.contains("container scaffold")
                                || content.contains("executable not yet wired")
                            {
                                offenders.push(format!(
                                    "tool={id}: apptainer def still contains scaffold placeholder text"
                                ));
                            }
                        }
                    }
                }
                other => offenders.push(format!("tool={id}: unsupported runtime `{other}`")),
            }
        }

        let version_cmd = as_str_field(entry, "version_cmd").unwrap_or("");
        if !version_cmd.contains("--version") {
            offenders.push(format!(
                "tool={id}: `version_cmd` must include --version for smoke contract"
            ));
        }

        let labels_required = entry
            .get("require_labels")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        if !labels_required {
            offenders.push(format!("tool={id}: require_labels must be true"));
        }
        if container_enabled && !is_planned {
            if runtimes.iter().any(|r| r == "docker")
                && as_str_field(entry, "dockerfile").unwrap_or("").is_empty()
            {
                offenders.push(format!(
                    "tool={id}: container=true docker runtime requires dockerfile"
                ));
            }
            if runtimes.iter().any(|r| r == "apptainer")
                && as_str_field(entry, "apptainer_def")
                    .unwrap_or("")
                    .is_empty()
            {
                offenders.push(format!(
                    "tool={id}: container=true apptainer runtime requires apptainer_def"
                ));
            }
        }
    }

    let docker_root = root.join("containers/docker");
    if docker_root.exists() {
        for entry in WalkDir::new(&docker_root)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if !entry.file_type().is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
                continue;
            };
            if !name.starts_with("Dockerfile.") {
                continue;
            }
            let tool = name.trim_start_matches("Dockerfile.").to_string();
            if !declared_docker_tool_files.contains(&tool) {
                offenders.push(format!(
                    "orphan dockerfile: {} (tool `{tool}` not present in registry)",
                    path.display()
                ));
            }
        }
    }

    let apptainer_root = root.join("containers/apptainer");
    if apptainer_root.exists() {
        for entry in WalkDir::new(&apptainer_root)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if !entry.file_type().is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
                continue;
            };
            if !name.ends_with(".def") {
                continue;
            }
            let tool = name.trim_end_matches(".def").to_string();
            if !declared_apptainer_tool_files.contains(&tool) {
                offenders.push(format!(
                    "orphan apptainer def: {} (tool `{tool}` not present in registry)",
                    path.display()
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tool registry completeness policy failures:\n{}",
        offenders.join("\n")
    );
}
