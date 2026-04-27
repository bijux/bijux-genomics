#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use bijux_dna_core::ids::DomainKind;
use std::collections::{BTreeMap, BTreeSet};
use support::workspace_root;

#[test]
fn policy__contracts__tool_registry_stage_domain_policy__production_tool_bindings_are_explicit_and_domain_consistent(
) {
    let registry_path = workspace_root().join("configs/ci/registry/tool_registry.toml");
    let raw = std::fs::read_to_string(&registry_path)
        .expect("read configs/ci/registry/tool_registry.toml");
    let parsed: toml::Value = raw.parse().expect("parse configs/ci/registry/tool_registry.toml");

    let tools = parsed.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let stages = parsed.get("stages").and_then(toml::Value::as_array).cloned().unwrap_or_default();

    let mut tool_stage_refs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for stage in &stages {
        let Some(stage_id) = stage.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        for key in ["primary_tools", "optional_alternatives", "validation_tools", "reporting_tools"]
        {
            for tool in list(stage, key) {
                tool_stage_refs.entry(tool).or_default().insert(stage_id.to_string());
            }
        }
    }

    let mut offenders = Vec::new();
    for tool in &tools {
        if tool.get("tool_id").and_then(toml::Value::as_str).is_none() {
            continue;
        }
        let id = tool.get("id").and_then(toml::Value::as_str).unwrap_or("<missing-id>");
        let declared_domain_raw =
            tool.get("domain").and_then(toml::Value::as_str).unwrap_or("").to_string();
        let declared_domain = DomainKind::try_from(declared_domain_raw.as_str()).ok();
        let declared_domains = list(tool, "domains").into_iter().collect::<BTreeSet<_>>();
        let declared_stage_ids = list(tool, "stage_ids");
        let declared_bindings = list(tool, "bindings");
        let status = tool.get("status").and_then(toml::Value::as_str).unwrap_or("supported");
        let discovered_stage_ids =
            tool_stage_refs.get(id).cloned().unwrap_or_default().into_iter().collect::<Vec<_>>();
        let effective_stage_ids = discovered_stage_ids
            .iter()
            .cloned()
            .chain(declared_stage_ids.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let binding_stage_ids = if declared_bindings.is_empty() {
            effective_stage_ids.clone()
        } else {
            declared_bindings
        };

        if declared_domain_raw.is_empty() {
            offenders.push(format!("tool={id}: missing `domain`"));
        } else if declared_domain.is_none() {
            offenders.push(format!("tool={id}: invalid `domain` value `{declared_domain_raw}`"));
        }
        if declared_stage_ids.is_empty() {
            offenders.push(format!("tool={id}: missing non-empty `stage_ids`"));
        }
        if binding_stage_ids.is_empty() {
            offenders.push(format!("tool={id}: missing non-empty `bindings`"));
        }
        if !support::registry_status_is_production(status) {
            continue;
        }
        if effective_stage_ids.is_empty() {
            offenders.push(format!(
                "tool={id}: referenced by no stage; every tool must appear in at least one stage"
            ));
            continue;
        }

        let stage_domain_set = binding_stage_ids
            .iter()
            .filter_map(|stage_id| {
                stage_id.split('.').next().and_then(|domain| DomainKind::try_from(domain).ok())
            })
            .collect::<BTreeSet<_>>();
        let stage_domain_strs = stage_domain_set
            .iter()
            .map(|domain| domain.as_str().to_string())
            .collect::<BTreeSet<_>>();

        if !declared_domains.is_empty() && stage_domain_strs != declared_domains {
            offenders.push(format!("tool={id}: declared domains {declared_domains:?} do not match discovered {stage_domain_strs:?}"));
        }
        if let Some(primary_domain) = declared_domain {
            if !stage_domain_set.contains(&primary_domain) {
                offenders.push(format!("tool={id}: declared primary domain `{declared_domain_raw}` is absent from bindings {binding_stage_ids:?}"));
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
            values.iter().filter_map(toml::Value::as_str).map(str::to_string).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
