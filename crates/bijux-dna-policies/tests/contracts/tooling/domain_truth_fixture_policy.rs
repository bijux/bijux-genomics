#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

fn configured_domains(root: &std::path::Path) -> Vec<String> {
    let path = root.join("configs").join("domains.toml");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("read domains config {}", path.display()));
    let parsed: toml::Value = raw
        .parse()
        .unwrap_or_else(|_| panic!("parse domains config {}", path.display()));
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

#[test]
fn policy__contracts__domain_truth_fixture_policy__supported_stage_tool_pairs_have_truth_fixtures()
{
    let root = support::workspace_root();
    let mut offenders = Vec::new();

    for domain in configured_domains(&root) {
        let index = root.join("domain").join(&domain).join("index.yaml");
        let raw = std::fs::read_to_string(&index)
            .unwrap_or_else(|_| panic!("read domain index {}", index.display()));

        let mut in_matrix = false;
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed == "stage_tool_compatibility:" {
                in_matrix = true;
                continue;
            }
            if in_matrix && !line.starts_with("  ") {
                in_matrix = false;
            }
            if !in_matrix || !trimmed.contains(':') || !trimmed.contains('[') {
                continue;
            }

            let mut parts = trimmed.splitn(2, ':');
            let Some(stage_id) = parts.next().map(str::trim) else {
                continue;
            };
            let Some(rhs) = parts.next().map(str::trim) else {
                continue;
            };
            let tools = rhs.trim_start_matches('[').trim_end_matches(']');
            for tool in tools
                .split(',')
                .map(str::trim)
                .map(|v| v.trim_matches('"').trim_matches('\''))
                .filter(|v| !v.is_empty())
            {
                let tool_file = root
                    .join("domain")
                    .join(&domain)
                    .join("tools")
                    .join(format!("{tool}.yaml"));
                let tool_raw = std::fs::read_to_string(&tool_file)
                    .unwrap_or_else(|_| panic!("read tool yaml {}", tool_file.display()));
                let status = tool_raw
                    .lines()
                    .find_map(|tool_line| {
                        let value = tool_line.trim();
                        value
                            .strip_prefix("status:")
                            .map(|status_value| status_value.trim().trim_matches('"').to_string())
                    })
                    .unwrap_or_else(|| "supported".to_string());
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

    assert!(
        offenders.is_empty(),
        "domain truth fixture policy violations:\n{}",
        offenders.join("\n")
    );
}
