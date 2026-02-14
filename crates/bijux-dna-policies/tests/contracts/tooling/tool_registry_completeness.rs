#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

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

#[test]
fn policy__contracts__tool_registry_completeness__registry_entries_are_machine_checkable() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();

    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
    ] {
        let path = root.join(rel);
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        let parsed: toml::Value = raw
            .parse()
            .unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));

        for entry in as_table_array(&parsed, "tools") {
            let id = as_str_field(entry, "id").unwrap_or("<missing>");
            for required in ["id", "version", "upstream", "pinned_commit"] {
                let value = as_str_field(entry, required).unwrap_or("").trim();
                if value.is_empty() {
                    offenders.push(format!("{rel}: tool={id} missing required field `{required}`"));
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tool registry completeness policy violations:\n{}",
        offenders.join("\n")
    );
}
