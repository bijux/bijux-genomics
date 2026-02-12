#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;

fn parse_registry_tools(path: &std::path::Path) -> Vec<toml::Value> {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let parsed: toml::Value = raw.parse().unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));
    parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
}

#[test]
fn policy__contracts__production_container_policy__production_tools_have_version_entry_and_smoke_contract(
) {
    let root = support::workspace_root();
    let registry_tools = parse_registry_tools(&root.join("configs/tool_registry.toml"));
    let versions_raw = std::fs::read_to_string(root.join("containers/versions/versions.toml"))
        .expect("read containers/versions/versions.toml");
    let versions: toml::Value = versions_raw.parse().expect("parse containers/versions/versions.toml");
    let version_table = versions
        .as_table()
        .expect("versions.toml top-level table");

    let mut offenders = Vec::new();
    let mut production_ids = BTreeSet::new();
    for tool in registry_tools {
        let status = tool
            .get("status")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        if status != "supported" {
            continue;
        }
        let id = tool
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        if id.is_empty() {
            continue;
        }
        production_ids.insert(id.clone());
        if !version_table.contains_key(&id) {
            offenders.push(format!(
                "production tool `{id}` missing [tool] entry in containers/versions/versions.toml"
            ));
        }

        let smoke_version = tool
            .get("smoke_version_cmd")
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .trim();
        let smoke_help = tool
            .get("smoke_help_cmd")
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .trim();
        let help_cmd = tool
            .get("help_cmd")
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .trim();
        let require_help = tool
            .get("smoke_require_help")
            .and_then(toml::Value::as_bool)
            .unwrap_or(true);
        let smoke_status = tool
            .get("smoke_status")
            .and_then(toml::Value::as_str)
            .unwrap_or("ok")
            .to_ascii_lowercase();

        if smoke_version.is_empty() {
            offenders.push(format!("production tool `{id}` missing smoke_version_cmd"));
        }
        if require_help && smoke_help.is_empty() && help_cmd.is_empty() {
            offenders.push(format!(
                "production tool `{id}` requires help smoke but lacks smoke_help_cmd/help_cmd"
            ));
        }
        if smoke_status == "warning" || smoke_status == "error" || smoke_status == "failed" {
            offenders.push(format!(
                "production tool `{id}` smoke_status must be successful, got `{smoke_status}`"
            ));
        }
    }

    bijux_dna_policies::policy_assert!(
        !production_ids.is_empty(),
        "no production tools found in configs/tool_registry.toml"
    );
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "production container policy failures:\n{}",
        offenders.join("\n")
    );
}

