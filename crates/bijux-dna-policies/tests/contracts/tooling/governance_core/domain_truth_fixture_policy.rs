#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

fn configured_domains(root: &std::path::Path) -> Vec<String> {
    let path = root.join("configs").join("ci").join("registry").join("domains.toml");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("read domains config {}", path.display()));
    let parsed: toml::Value =
        raw.parse().unwrap_or_else(|_| panic!("parse domains config {}", path.display()));
    parsed
        .get("domains")
        .and_then(toml::Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry.get("id").and_then(toml::Value::as_str))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_yaml(path: &std::path::Path) -> serde_yaml::Value {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_yaml::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

fn yaml_string(value: &serde_yaml::Value, key: &str) -> Option<String> {
    value.get(key).and_then(serde_yaml::Value::as_str).map(str::to_string)
}

#[test]
fn policy__contracts__domain_truth_fixture_policy__supported_stage_tool_pairs_have_truth_fixtures()
{
    let root = support::workspace_root();
    let mut offenders = Vec::new();

    for domain in configured_domains(&root) {
        let index = root.join("domain").join(&domain).join("index.yaml");
        let parsed = if std::fs::read_to_string(&index).is_ok() {
            parse_yaml(&index)
        } else {
            offenders.push(format!("missing domain index for {domain}: {}", index.display()));
            continue;
        };

        let Some(stage_tool_compatibility) =
            parsed.get("stage_tool_compatibility").and_then(serde_yaml::Value::as_mapping)
        else {
            offenders.push(format!(
                "missing stage_tool_compatibility map for {domain}: {}",
                index.display()
            ));
            continue;
        };

        for (stage_key, tools_value) in stage_tool_compatibility {
            let Some(stage_id) = stage_key.as_str() else {
                offenders.push(format!(
                    "non-string stage_tool_compatibility key in {}",
                    index.display()
                ));
                continue;
            };
            let Some(tools) = tools_value.as_sequence() else {
                continue;
            };

            for tool in tools.iter().filter_map(serde_yaml::Value::as_str) {
                let tool_file =
                    root.join("domain").join(&domain).join("tools").join(format!("{tool}.yaml"));
                let tool_doc = if std::fs::read_to_string(&tool_file).is_ok() {
                    parse_yaml(&tool_file)
                } else {
                    offenders.push(format!(
                        "missing tool yaml for {domain} {stage_id} / {tool}: {}",
                        tool_file.display()
                    ));
                    continue;
                };
                let status =
                    yaml_string(&tool_doc, "status").unwrap_or_else(|| "supported".to_string());
                if status != "supported" {
                    continue;
                }

                let fixture = root
                    .join("domain")
                    .join(&domain)
                    .join("fixtures")
                    .join(stage_id)
                    .join(format!("{tool}.txt"));
                if !fixture.exists() {
                    offenders.push(format!(
                        "missing truth fixture for {domain} {stage_id} / {tool}: {}",
                        fixture.display()
                    ));
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "supported stage/tool pairs must keep truth fixtures:\n{}",
        offenders.join("\n")
    );
}
