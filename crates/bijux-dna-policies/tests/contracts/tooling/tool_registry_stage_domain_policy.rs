#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use bijux_dna_core::ids::DomainKind;
use std::collections::{BTreeMap, BTreeSet};
use support::workspace_root;

#[test]
fn policy__contracts__tool_registry_stage_domain_policy__each_tool_has_exactly_one_domain_and_stage_binding(
) {
    let registry_path = workspace_root().join("configs/tool_registry.toml");
    let raw = std::fs::read_to_string(&registry_path).expect("read configs/tool_registry.toml");
    let parsed: toml::Value = raw.parse().expect("parse configs/tool_registry.toml");

    let tools = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let stages = parsed
        .get("stages")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut tool_stage_refs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for stage in &stages {
        let Some(stage_id) = stage.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        for key in [
            "primary_tools",
            "optional_alternatives",
            "validation_tools",
            "reporting_tools",
        ] {
            for tool in list(stage, key) {
                tool_stage_refs
                    .entry(tool)
                    .or_default()
                    .insert(stage_id.to_string());
            }
        }
    }

    let mut offenders = Vec::new();
    let cross_domain_allowlist = ["multiqc", "samtools"];
    for tool in &tools {
        if tool.get("tool_id").and_then(toml::Value::as_str).is_none() {
            continue;
        }
        let id = tool
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<missing-id>");
        let declared_domain_raw = tool
            .get("domain")
            .and_then(toml::Value::as_str)
            .unwrap_or("")
            .to_string();
        let declared_domain = DomainKind::try_from(declared_domain_raw.as_str()).ok();
        let declared_domains = list(tool, "domains").into_iter().collect::<BTreeSet<_>>();
        let declared_stage_ids = list(tool, "stage_ids");
        let status = tool
            .get("status")
            .and_then(toml::Value::as_str)
            .unwrap_or("supported");
        let discovered_stage_ids = tool_stage_refs
            .get(id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        let effective_stage_ids = discovered_stage_ids
            .iter()
            .cloned()
            .chain(declared_stage_ids.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        if declared_domain_raw.is_empty() {
            offenders.push(format!("tool={id}: missing `domain`"));
        } else if declared_domain.is_none() {
            offenders.push(format!(
                "tool={id}: invalid `domain` value `{declared_domain_raw}`"
            ));
        }
        if declared_stage_ids.is_empty() {
            offenders.push(format!("tool={id}: missing non-empty `stage_ids`"));
        }
        if status != "supported" {
            continue;
        }
        if effective_stage_ids.is_empty() {
            offenders.push(format!(
                "tool={id}: referenced by no stage; every tool must appear in at least one stage"
            ));
            continue;
        }

        let stage_domain_set = effective_stage_ids
            .iter()
            .filter_map(|stage_id| {
                stage_id
                    .split('.')
                    .next()
                    .and_then(|domain| DomainKind::try_from(domain).ok())
            })
            .collect::<BTreeSet<_>>();
        let stage_domain_strs = stage_domain_set
            .iter()
            .map(|domain| domain.as_str().to_string())
            .collect::<BTreeSet<_>>();

        if !declared_domains.is_empty() {
            if stage_domain_strs != declared_domains {
                offenders.push(format!(
                    "tool={id}: declared domains {:?} do not match discovered {:?}",
                    declared_domains, stage_domain_strs
                ));
            }
            continue;
        }

        if stage_domain_set.len() != 1 && !cross_domain_allowlist.contains(&id) {
            offenders.push(format!(
                "tool={id}: must map to exactly one domain; found {:?}",
                stage_domain_strs
            ));
        } else if stage_domain_set.len() == 1 {
            if let Some(actual_domain) = stage_domain_set.iter().next() {
                if Some(*actual_domain) != declared_domain {
                    offenders.push(format!(
                        "tool={id}: declared domain `{declared_domain_raw}` does not match discovered `{}`",
                        actual_domain.as_str()
                    ));
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "tool domain/stage binding policy violations:\n{}",
        offenders.join("\n")
    );
}

fn list(table: &toml::Value, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
