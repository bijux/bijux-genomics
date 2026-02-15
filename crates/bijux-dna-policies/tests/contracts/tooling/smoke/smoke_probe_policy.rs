#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__smoke_probe_policy__supported_tools_define_valid_probe_contract() {
    let root = support::workspace_root();
    let raw = std::fs::read_to_string(root.join("configs/ci/registry/tool_registry.toml"))
        .expect("read configs/ci/registry/tool_registry.toml");
    let parsed: toml::Value = raw
        .parse()
        .expect("parse configs/ci/registry/tool_registry.toml");
    let tools = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut offenders = Vec::new();
    for tool in tools {
        let id = tool
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<missing-id>");
        let status = tool
            .get("status")
            .and_then(toml::Value::as_str)
            .unwrap_or("supported");
        if status != "supported" {
            continue;
        }
        let version_cmd = tool
            .get("smoke_version_cmd")
            .and_then(toml::Value::as_str)
            .or_else(|| tool.get("version_cmd").and_then(toml::Value::as_str))
            .unwrap_or("")
            .trim();
        let help_cmd = tool
            .get("smoke_help_cmd")
            .and_then(toml::Value::as_str)
            .or_else(|| tool.get("help_cmd").and_then(toml::Value::as_str))
            .unwrap_or("")
            .trim();
        let require_help = tool
            .get("smoke_require_help")
            .and_then(toml::Value::as_bool)
            .unwrap_or(true);

        if version_cmd.is_empty() {
            offenders.push(format!("tool={} missing smoke version probe", id));
        }
        if require_help && help_cmd.is_empty() {
            offenders.push(format!(
                "tool={} requires help probe but no smoke_help_cmd/help_cmd defined",
                id
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "smoke probe policy violations:\n{}",
        offenders.join("\n")
    );
}
