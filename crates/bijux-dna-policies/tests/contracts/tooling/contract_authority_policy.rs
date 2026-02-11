#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};

fn table_array<'a>(root: &'a toml::Value, key: &str) -> Vec<&'a toml::Value> {
    root.get(key)
        .and_then(toml::Value::as_array)
        .map(|rows| rows.iter().collect())
        .unwrap_or_default()
}

fn param_rows<'a>(root: &'a toml::Value) -> Vec<&'a toml::Value> {
    let rows = table_array(root, "params");
    if rows.is_empty() {
        table_array(root, "entries")
    } else {
        rows
    }
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

fn parse_toml(path: &std::path::Path) -> toml::Value {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    raw.parse::<toml::Value>()
        .unwrap_or_else(|_| panic!("parse {}", path.display()))
}

#[test]
fn policy__contracts__contract_authority_policy__param_schema_ids_are_not_hardcoded_outside_domain()
{
    let root = support::workspace_root();
    let pattern = regex::Regex::new(
        r#"bijux\.[a-z0-9_]+\.(call|filter|stats|fastq|bam)?\.?params\.[a-z0-9_.-]*v[0-9]+"#,
    )
    .expect("compile regex");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let path_str = path.display().to_string();
        if path_str.contains("crates/bijux-dna-domain-")
            || path_str.contains("crates/bijux-dna-pipelines/tests")
        {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if pattern.is_match(&raw) {
            offenders.push(path_str);
        }
    }
    assert!(
        offenders.is_empty(),
        "param schema ids must come from configs/param_registry*.toml, not hardcoded in consumer code:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__contract_authority_policy__stage_contracts_are_complete_per_domain_policy() {
    let root = support::workspace_root();
    let domains = parse_toml(&root.join("configs/domains.toml"));
    let images = parse_toml(&root.join("configs/images.toml"));
    let image_ids = images
        .as_table()
        .map(|table| table.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    let mut offenders = Vec::new();

    for domain in table_array(&domains, "domains") {
        let id = domain
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<unknown>");
        let experimental = domain
            .get("experimental")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        let stages_ssot = domain
            .get("stages_ssot")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        let tool_registry_ssot = domain
            .get("tool_registry_ssot")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        let param_registry_ssot = domain
            .get("param_registry_ssot")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        if stages_ssot.is_empty() || tool_registry_ssot.is_empty() || param_registry_ssot.is_empty()
        {
            offenders.push(format!(
                "domain {id}: missing stages/tool/param ssot pointers in configs/domains.toml"
            ));
            continue;
        }
        let stages = parse_toml(&root.join(stages_ssot));
        let registry = parse_toml(&root.join(tool_registry_ssot));
        let params = parse_toml(&root.join(param_registry_ssot));

        let param_stage_ids = param_rows(&params)
            .into_iter()
            .filter_map(|row| row.get("stage_id").and_then(toml::Value::as_str))
            .map(str::to_string)
            .collect::<BTreeSet<_>>();

        let mut tools_by_stage = BTreeMap::<String, BTreeSet<String>>::new();
        let mut tool_metrics = BTreeMap::<String, String>::new();
        for tool in table_array(&registry, "tools") {
            let tool_id = tool
                .get("id")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let metrics_schema = tool
                .get("metrics_schema")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_string();
            tool_metrics.insert(tool_id.clone(), metrics_schema);
            for stage_id in list(tool, "stage_ids") {
                tools_by_stage
                    .entry(stage_id)
                    .or_default()
                    .insert(tool_id.clone());
            }
        }

        for stage in table_array(&stages, "stages") {
            let stage_id = stage
                .get("id")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unknown>");
            if !stage_id.starts_with(&format!("{id}.")) {
                continue;
            }
            let status = stage
                .get("status")
                .and_then(toml::Value::as_str)
                .unwrap_or("supported");
            if status != "supported" {
                continue;
            }

            let has_param = param_stage_ids.contains(stage_id);
            let stage_metrics_schema = stage
                .get("metrics_schema")
                .and_then(toml::Value::as_str)
                .unwrap_or("");
            let stage_tools = list(stage, "tools")
                .into_iter()
                .chain(
                    tools_by_stage
                        .get(stage_id)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter(),
                )
                .collect::<BTreeSet<_>>();
            let has_metrics = !stage_metrics_schema.trim().is_empty()
                || stage_metrics_schema == "none"
                || stage_tools.iter().any(|tool| {
                    tool_metrics
                        .get(tool)
                        .is_some_and(|schema| !schema.trim().is_empty() && schema != "bijux.unknown.v1")
                });
            let has_tools = !stage_tools.is_empty();
            let runnable = status == "supported";
            let has_images = stage_tools.iter().all(|tool| image_ids.contains(tool));

            if !experimental {
                if !has_param {
                    offenders.push(format!(
                        "domain={id} stage={stage_id}: missing param registry entry in {param_registry_ssot}"
                    ));
                }
                if !has_metrics {
                    offenders.push(format!(
                        "domain={id} stage={stage_id}: missing metrics schema or explicit `none` in {stages_ssot}"
                    ));
                }
                if !has_tools {
                    offenders.push(format!(
                        "domain={id} stage={stage_id}: missing tool binding in {stages_ssot}/{tool_registry_ssot}"
                    ));
                }
                if runnable && !has_images {
                    offenders.push(format!(
                        "domain={id} stage={stage_id}: missing image binding for at least one bound tool in configs/images.toml"
                    ));
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "contract authority completeness failures:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__contract_authority_policy__registry_unknowns_images_and_required_tools_do_not_drift(
) {
    let root = support::workspace_root();
    let production_registries = [
        "configs/tool_registry.toml",
        "configs/tool_registry_vcf.toml",
    ];
    let all_registries = [
        "configs/tool_registry.toml",
        "configs/tool_registry_vcf.toml",
        "configs/tool_registry_experimental.toml",
    ];
    let images = parse_toml(&root.join("configs/images.toml"));
    let image_ids = images
        .as_table()
        .map(|table| table.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    let mut offenders = Vec::new();
    let mut production_supported_tools = BTreeSet::new();
    let mut all_registry_tools = BTreeSet::new();

    for rel in production_registries {
        let registry = parse_toml(&root.join(rel));
        for tool in table_array(&registry, "tools") {
            let id = tool
                .get("id")
                .and_then(toml::Value::as_str)
                .unwrap_or("<missing-id>");
            let status = tool
                .get("status")
                .and_then(toml::Value::as_str)
                .unwrap_or("supported");
            let upstream = tool
                .get("upstream")
                .and_then(toml::Value::as_str)
                .unwrap_or_default();
            if upstream == "unknown" {
                offenders.push(format!("{rel}: tool {id} has upstream=unknown"));
            }
            if status == "supported" {
                production_supported_tools.insert(id.to_string());
                if !image_ids.contains(id) {
                    offenders.push(format!(
                        "{rel}: supported tool {id} missing image catalog entry in configs/images.toml"
                    ));
                }
            }
        }
    }
    for rel in all_registries {
        let registry = parse_toml(&root.join(rel));
        for tool in table_array(&registry, "tools") {
            if let Some(id) = tool.get("id").and_then(toml::Value::as_str) {
                all_registry_tools.insert(id.to_string());
            }
        }
    }

    let required = parse_toml(&root.join("configs/required_tools.toml"));
    let required_vcf = parse_toml(&root.join("configs/required_tools_vcf.toml"));
    let required_tools = list(&required, "required_tools")
        .into_iter()
        .chain(list(&required_vcf, "required_tools"))
        .collect::<BTreeSet<_>>();

    let missing_required = required_tools
        .iter()
        .filter(|tool| !all_registry_tools.contains(*tool))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_required.is_empty() {
        offenders.push(format!(
            "required_tools drift: missing from production registries: {:?}",
            missing_required
        ));
    }

    assert!(
        offenders.is_empty(),
        "registry/image/required_tools authority failures:\n{}",
        offenders.join("\n")
    );
}
