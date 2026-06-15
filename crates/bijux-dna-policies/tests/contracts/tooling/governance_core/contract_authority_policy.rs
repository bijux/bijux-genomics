#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};

fn table_array<'a>(root: &'a toml::Value, key: &str) -> Vec<&'a toml::Value> {
    root.get(key)
        .and_then(toml::Value::as_array)
        .map(|rows| rows.iter().collect())
        .unwrap_or_default()
}

fn param_rows(root: &toml::Value) -> Vec<&toml::Value> {
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
            values.iter().filter_map(toml::Value::as_str).map(str::to_string).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_toml(path: &std::path::Path) -> toml::Value {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    raw.parse::<toml::Value>().unwrap_or_else(|_| panic!("parse {}", path.display()))
}

fn resolve_ssot_path(root: &std::path::Path, rel: &str) -> std::path::PathBuf {
    let direct = root.join(rel);
    if direct.exists() {
        return direct;
    }
    if let Some(stripped) = rel.strip_prefix("configs/") {
        let ci = root.join("configs/ci").join(stripped);
        if ci.exists() {
            return ci;
        }
    }
    direct
}

#[test]
fn policy__contracts__contract_authority_policy__param_schema_ids_are_not_hardcoded_outside_domain()
{
    let root = support::workspace_root();
    let pattern = regex::Regex::new(
        r"bijux\.[a-z0-9_]+\.(call|filter|stats|fastq|bam)?\.?params\.[a-z0-9_.-]*v[0-9]+",
    )
    .expect("compile regex");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
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
        "param schema ids must come from configs/ci/param_registry*.toml, not hardcoded in consumer code:\n{}",
        offenders.join("\n")
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn policy__contracts__contract_authority_policy__stage_contracts_are_complete_per_domain_policy() {
    let root = support::workspace_root();
    let domains = parse_toml(&root.join("configs/ci/registry/domains.toml"));
    let mut offenders = Vec::new();

    for domain in table_array(&domains, "domains") {
        let id = domain.get("id").and_then(toml::Value::as_str).unwrap_or("<unknown>");
        let experimental =
            domain.get("experimental").and_then(toml::Value::as_bool).unwrap_or(false);
        let stages_ssot =
            domain.get("stages_ssot").and_then(toml::Value::as_str).unwrap_or_default();
        let tool_registry_ssot =
            domain.get("tool_registry_ssot").and_then(toml::Value::as_str).unwrap_or_default();
        let param_registry_ssot =
            domain.get("param_registry_ssot").and_then(toml::Value::as_str).unwrap_or_default();
        if stages_ssot.is_empty() || tool_registry_ssot.is_empty() || param_registry_ssot.is_empty()
        {
            offenders.push(format!(
                "domain {id}: missing stages/tool/param ssot pointers in configs/ci/registry/domains.toml"
            ));
            continue;
        }
        let stages_path = resolve_ssot_path(&root, stages_ssot);
        let registry_path = resolve_ssot_path(&root, tool_registry_ssot);
        let params_path = resolve_ssot_path(&root, param_registry_ssot);
        if !(stages_path.exists() && registry_path.exists() && params_path.exists()) {
            offenders.push(format!(
                "domain {id}: contract authority ssot pointer missing: stages={}, registry={}, params={}",
                stages_path.display(),
                registry_path.display(),
                params_path.display()
            ));
            continue;
        }
        let stages = parse_toml(&stages_path);
        let registry = parse_toml(&registry_path);
        let params = parse_toml(&params_path);

        let param_stage_ids = param_rows(&params)
            .into_iter()
            .filter_map(|row| row.get("stage_id").and_then(toml::Value::as_str))
            .map(str::to_string)
            .collect::<BTreeSet<_>>();

        let mut tools_by_stage = BTreeMap::<String, BTreeSet<String>>::new();
        let mut tool_metrics = BTreeMap::<String, String>::new();
        for tool in table_array(&registry, "tools") {
            let tool_id =
                tool.get("id").and_then(toml::Value::as_str).unwrap_or_default().to_string();
            let status = tool.get("status").and_then(toml::Value::as_str).unwrap_or("supported");
            if !support::registry_status_is_production(status) {
                continue;
            }
            let metrics_schema = tool
                .get("metrics_schema")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_string();
            tool_metrics.insert(tool_id.clone(), metrics_schema);
            for stage_id in list(tool, "stage_ids") {
                tools_by_stage.entry(stage_id).or_default().insert(tool_id.clone());
            }
        }

        for stage in table_array(&stages, "stages") {
            let stage_id = stage.get("id").and_then(toml::Value::as_str).unwrap_or("<unknown>");
            if !stage_id.starts_with(&format!("{id}.")) {
                continue;
            }
            let status = stage.get("status").and_then(toml::Value::as_str).unwrap_or("supported");
            if status != "supported" {
                continue;
            }

            let has_param = param_stage_ids.contains(stage_id);
            let stage_metrics_schema =
                stage.get("metrics_schema").and_then(toml::Value::as_str).unwrap_or("");
            let stage_tools = list(stage, "tools")
                .into_iter()
                .chain(tools_by_stage.get(stage_id).cloned().unwrap_or_default().into_iter())
                .collect::<BTreeSet<_>>();
            let has_metrics = !stage_metrics_schema.trim().is_empty()
                || stage_metrics_schema == "none"
                || stage_tools.iter().any(|tool| {
                    tool_metrics.get(tool).is_some_and(|schema| {
                        !schema.trim().is_empty() && schema != "bijux.unknown.v1"
                    })
                });
            let has_tools = !stage_tools.is_empty();

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
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "domain stage contracts must remain complete per authority policy:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__contract_authority_policy__registry_unknowns_images_and_required_tools_do_not_drift(
) {
    let root = support::workspace_root();
    let production_registries =
        ["configs/ci/registry/tool_registry.toml", "configs/ci/registry/tool_registry_vcf.toml"];
    let all_registries = [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
    ];
    let images = parse_toml(&root.join("configs/ci/tools/images.toml"));
    let image_ids = images
        .as_table()
        .map(|table| table.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    let mut offenders = Vec::new();
    let mut all_registry_tools = BTreeSet::new();

    for rel in production_registries {
        let registry = parse_toml(&root.join(rel));
        for tool in table_array(&registry, "tools") {
            let id = tool.get("id").and_then(toml::Value::as_str).unwrap_or("<missing-id>");
            let status = tool.get("status").and_then(toml::Value::as_str).unwrap_or("supported");
            let upstream = tool.get("upstream").and_then(toml::Value::as_str).unwrap_or_default();
            if upstream == "unknown" {
                offenders.push(format!("{rel}: tool {id} has upstream=unknown"));
            }
            if support::registry_status_is_production(status) && !image_ids.contains(id) {
                offenders.push(format!(
                    "{rel}: production tool {id} missing image catalog entry in configs/ci/tools/images.toml"
                ));
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

    let required = parse_toml(&root.join("configs/ci/tools/required_tools.toml"));
    let required_vcf = parse_toml(&root.join("configs/ci/tools/required_tools_vcf.toml"));
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
            "required_tools drift: missing from production registries: {missing_required:?}"
        ));
    }

    assert!(
        offenders.is_empty(),
        "registry/image/required_tools authority failures:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__contract_authority_policy__production_stages_have_scientific_rationale() {
    let root = support::workspace_root();
    let stages = parse_toml(&root.join("configs/ci/stages/stages.toml"));
    let registry = parse_toml(&root.join("configs/ci/registry/tool_registry.toml"));
    let rationale_by_stage = table_array(&registry, "stages")
        .into_iter()
        .filter_map(|row| {
            let stage_id = row.get("id").and_then(toml::Value::as_str)?;
            let rationale = row
                .get("default_rationale")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            Some((stage_id.to_string(), rationale))
        })
        .collect::<BTreeMap<_, _>>();

    let mut offenders = Vec::new();
    for stage in table_array(&stages, "stages") {
        let stage_id = stage.get("id").and_then(toml::Value::as_str).unwrap_or("<unknown>");
        let status = stage.get("status").and_then(toml::Value::as_str).unwrap_or("supported");
        let experimental =
            stage.get("experimental").and_then(toml::Value::as_bool).unwrap_or(false);
        if status != "supported" || experimental {
            continue;
        }
        if rationale_by_stage.get(stage_id).is_none_or(|value| value.trim().is_empty()) {
            offenders.push(format!(
                "production stage {stage_id} has no non-empty default_rationale in configs/ci/registry/tool_registry.toml [[stages]]"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "production scientific rationale failures:\n{}",
        offenders.join("\n")
    );
}
