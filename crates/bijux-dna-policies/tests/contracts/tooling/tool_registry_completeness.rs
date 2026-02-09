#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use support::workspace_root;

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

#[test]
fn policy__contracts__tool_registry_completeness__registry_entries_are_machine_checkable() {
    let root = workspace_root();
    let registry_path = root.join("configs/tool_registry.toml");
    let raw = std::fs::read_to_string(&registry_path).expect("read configs/tool_registry.toml");
    let parsed: toml::Value = raw.parse().expect("parse configs/tool_registry.toml");
    let tools = as_table_array(&parsed, "tools");
    let mut offenders = Vec::new();

    if tools.is_empty() {
        offenders.push("configs/tool_registry.toml: missing [[tools]] entries".to_string());
    }

    for entry in tools {
        let id = as_str_field(entry, "id").unwrap_or("<missing>");
        for required in ["id", "version", "upstream", "pinned_commit", "version_cmd"] {
            let value = as_str_field(entry, required).unwrap_or("");
            if value.trim().is_empty() {
                offenders.push(format!("tool={id}: missing required field `{required}`"));
            }
        }

        let runtimes = runtimes(entry);
        if runtimes.is_empty() {
            offenders.push(format!("tool={id}: `runtimes` must be non-empty"));
        }

        for runtime in &runtimes {
            match runtime.as_str() {
                "docker" => {
                    let path = as_str_field(entry, "dockerfile").unwrap_or("");
                    if path.is_empty() {
                        offenders.push(format!("tool={id}: missing dockerfile path"));
                    } else {
                        let abs = root.join(path);
                        if !abs.exists() {
                            offenders.push(format!("tool={id}: dockerfile not found at {path}"));
                        } else {
                            let content = std::fs::read_to_string(&abs).unwrap_or_default();
                            if !content.contains("SPDX-License-Identifier: GPL-3.0-only") {
                                offenders.push(format!("tool={id}: dockerfile missing GPL SPDX header"));
                            }
                            if !content.contains("org.opencontainers.image.licenses=\"GPL-3.0\"") {
                                offenders.push(format!(
                                    "tool={id}: dockerfile missing OCI GPL-3.0 license label"
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
                        let abs = root.join(path);
                        if !abs.exists() {
                            offenders.push(format!("tool={id}: apptainer def not found at {path}"));
                        } else {
                            let content = std::fs::read_to_string(&abs).unwrap_or_default();
                            if !content.contains("SPDX-License-Identifier: GPL-3.0-only") {
                                offenders.push(format!("tool={id}: apptainer def missing GPL SPDX header"));
                            }
                            if !content.contains("org.opencontainers.image.licenses GPL-3.0") {
                                offenders.push(format!(
                                    "tool={id}: apptainer def missing OCI GPL-3.0 license label"
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
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tool registry completeness policy failures:\n{}",
        offenders.join("\n")
    );
}
