#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use bijux_dna_stage_contract::{entries, ReadinessBadge};

fn read_toml(path: &Path) -> toml::Value {
    std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .parse()
        .unwrap_or_else(|err| panic!("parse {}: {err}", path.display()))
}

fn stage_rows(path: &Path) -> Vec<toml::Value> {
    read_toml(path).get("stages").and_then(toml::Value::as_array).cloned().unwrap_or_default()
}

fn stage_ids_from_configs(root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for rel in [
        "configs/ci/stages/stages.toml",
        "configs/ci/stages/stages_vcf.toml",
        "configs/ci/stages/stages_vcf_downstream.toml",
    ] {
        for row in stage_rows(&root.join(rel)) {
            if let Some(id) = row.get("id").and_then(toml::Value::as_str) {
                out.insert(id.to_string());
            }
        }
    }
    out
}

fn config_stage_rows_by_id(root: &Path) -> BTreeMap<String, toml::Value> {
    let mut out = BTreeMap::new();
    for rel in [
        "configs/ci/stages/stages.toml",
        "configs/ci/stages/stages_vcf.toml",
        "configs/ci/stages/stages_vcf_downstream.toml",
    ] {
        for row in stage_rows(&root.join(rel)) {
            if let Some(id) = row.get("id").and_then(toml::Value::as_str) {
                out.insert(id.to_string(), row.clone());
            }
        }
    }
    out
}

fn domain_stage_docs(root: &Path) -> BTreeMap<String, serde_yaml::Value> {
    let mut out = BTreeMap::new();
    for domain in ["fastq", "bam", "vcf"] {
        let dir = root.join("domain").join(domain).join("stages");
        let iter = std::fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("read_dir {}: {err}", dir.display()));
        for entry in iter.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|s| s.to_str()).is_some_and(|n| n.starts_with('_')) {
                continue;
            }
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
            let doc: serde_yaml::Value = serde_yaml::from_str(&raw)
                .unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));
            let id = doc
                .get("stage_id")
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_else(|| panic!("missing stage_id in {}", path.display()))
                .to_string();
            out.insert(id, doc);
        }
    }
    out
}

fn yaml_str<'a>(v: &'a serde_yaml::Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(serde_yaml::Value::as_str)
}

fn yaml_list(v: &serde_yaml::Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(serde_yaml::Value::as_sequence)
        .map(|xs| {
            xs.iter().filter_map(serde_yaml::Value::as_str).map(str::to_string).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn yaml_metric_keys(v: &serde_yaml::Value) -> Vec<String> {
    v.get("metrics")
        .and_then(serde_yaml::Value::as_sequence)
        .map(|xs| {
            xs.iter()
                .filter_map(|entry| {
                    if let Some(name) = entry.get("name").and_then(serde_yaml::Value::as_str) {
                        Some(name.to_string())
                    } else {
                        entry.as_str().map(str::to_string)
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[test]
fn policy__contracts__stage_executor_parity_policy__registry_and_ssot_are_consistent() {
    let root = support::workspace_root();
    let domain_docs = domain_stage_docs(&root);
    let _config_stage_ids = stage_ids_from_configs(&root);

    let deprecations = read_toml(&root.join("configs/ci/registry/deprecations.toml"));
    let deprecated_stage_ids = deprecations
        .get("deprecations")
        .and_then(toml::Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|r| r.get("stage").and_then(toml::Value::as_str))
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let mut tool_rows = Vec::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let parsed = read_toml(&root.join(rel));
        tool_rows.extend(
            parsed.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default(),
        );
    }
    let tools_by_id = tool_rows
        .iter()
        .filter_map(|row| {
            row.get("id").and_then(toml::Value::as_str).map(|id| (id.to_string(), row))
        })
        .fold(BTreeMap::<String, Vec<&toml::Value>>::new(), |mut acc, (id, row)| {
            acc.entry(id).or_default().push(row);
            acc
        });
    let config_rows = config_stage_rows_by_id(&root);

    let executors =
        entries().iter().map(|e| (e.stage_id.to_string(), e)).collect::<BTreeMap<_, _>>();
    let mut offenders = Vec::new();

    for (stage_id, doc) in &domain_docs {
        let status = yaml_str(doc, "status").unwrap_or("planned");
        let out_of_scope = status == "out_of_scope"
            || !yaml_list(doc, "planned_out_of_scope").is_empty()
            || deprecated_stage_ids.contains(stage_id);
        let has_executor = executors.contains_key(stage_id);
        if status == "supported" && !has_executor && !out_of_scope {
            offenders.push(format!(
                "supported stage {stage_id} missing code-backed executor (not deprecated/out_of_scope)"
            ));
        }
        let stage_tools = config_rows
            .get(stage_id)
            .and_then(|row| row.get("tools"))
            .and_then(toml::Value::as_array)
            .map(|xs| {
                xs.iter().filter_map(toml::Value::as_str).map(str::to_string).collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if status == "supported" && stage_tools.is_empty() {
            offenders.push(format!(
                "supported stage {stage_id} must declare required tools in configs/ci/stages*.toml"
            ));
        }
        let metrics_schema_config = config_rows
            .get(stage_id)
            .and_then(|row| row.get("metrics_schema"))
            .and_then(toml::Value::as_str)
            .unwrap_or("");
        let metrics_schema_domain = yaml_str(doc, "metrics_schema").unwrap_or("");
        if status == "supported"
            && metrics_schema_config.trim().is_empty()
            && metrics_schema_domain.trim().is_empty()
        {
            offenders.push(format!(
                "supported stage {stage_id} must declare metrics_schema (domain yaml or configs/ci/stages*.toml)"
            ));
        }
        let metrics = yaml_metric_keys(doc);
        if status == "supported" && metrics.is_empty() {
            offenders.push(format!("supported stage {stage_id} must declare metrics keys"));
        }
        if status == "out_of_scope" {
            if yaml_list(doc, "planned_out_of_scope").is_empty() {
                offenders.push(format!(
                    "out_of_scope stage {stage_id} must document refusal rationale in planned_out_of_scope"
                ));
            }
            if !deprecated_stage_ids.contains(stage_id) {
                offenders.push(format!(
                    "out_of_scope stage {stage_id} must be listed in configs/ci/registry/deprecations.toml"
                ));
            }
        }
        for tool_id in stage_tools {
            if status != "supported" {
                continue;
            }
            let Some(tool_rows) = tools_by_id.get(&tool_id) else {
                offenders.push(format!(
                    "stage {stage_id} references tool {tool_id} missing from tool registry"
                ));
                continue;
            };
            let tool = tool_rows
                .iter()
                .copied()
                .find(|row| {
                    row.get("stage_ids").and_then(toml::Value::as_array).is_some_and(|stage_ids| {
                        stage_ids.iter().any(|value| value.as_str() == Some(stage_id))
                    })
                })
                .unwrap_or(tool_rows[0]);
            let runtimes = tool
                .get("runtimes")
                .and_then(toml::Value::as_array)
                .map(|xs| xs.iter().filter_map(toml::Value::as_str).collect::<BTreeSet<_>>())
                .unwrap_or_default();
            let host_only =
                doc.get("host_only").and_then(serde_yaml::Value::as_bool).unwrap_or(false);
            let host_only_justification = yaml_str(doc, "host_only_justification").unwrap_or("");
            if !(host_only || runtimes.contains("docker") && runtimes.contains("apptainer")) {
                offenders.push(format!(
                    "stage {stage_id} tool {tool_id} must provide docker+apptainer runtimes"
                ));
            }
            if host_only && host_only_justification.trim().is_empty() {
                offenders.push(format!(
                    "stage {stage_id} host_only=true requires host_only_justification"
                ));
            }
        }
    }

    for stage_id in executors.keys() {
        if !domain_docs.contains_key(stage_id) {
            offenders.push(format!(
                "executor exists for hidden stage {stage_id} (missing domain/*/stages/*.yaml)"
            ));
        }
    }

    let domain_stage_ids = domain_docs
        .iter()
        .filter(|(_, doc)| yaml_str(doc, "status").unwrap_or("planned") == "supported")
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    let config_supported_ids = config_rows
        .iter()
        .filter(|(_, row)| row.get("status").and_then(toml::Value::as_str) == Some("supported"))
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    let missing_in_configs =
        domain_stage_ids.difference(&config_supported_ids).cloned().collect::<Vec<_>>();
    let extra_in_configs =
        config_supported_ids.difference(&domain_stage_ids).cloned().collect::<Vec<_>>();
    if !missing_in_configs.is_empty() {
        offenders
            .push(format!("configs/ci/stages*.toml missing domain stages: {missing_in_configs:?}"));
    }
    if !extra_in_configs.is_empty() {
        offenders
            .push(format!("configs/ci/stages*.toml has non-domain stages: {extra_in_configs:?}"));
    }

    let stable_stage_ids = stage_rows(&root.join("configs/ci/stages/stages.toml"))
        .into_iter()
        .filter(|row| row.get("status").and_then(toml::Value::as_str) == Some("supported"))
        .filter_map(|row| row.get("id").and_then(toml::Value::as_str).map(str::to_string))
        .collect::<Vec<_>>();
    for stage_id in stable_stage_ids {
        let Some(executor) = executors.get(&stage_id) else {
            offenders.push(format!("stable profile stage {stage_id} missing code-backed executor"));
            continue;
        };
        if !matches!(executor.readiness, ReadinessBadge::Supported | ReadinessBadge::Certified) {
            offenders.push(format!(
                "stable profile stage {stage_id} has non-stable readiness badge {:?}",
                executor.readiness
            ));
        }
    }

    let always_refuse_markers = walkdir::WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .filter_map(|e| {
            let raw = std::fs::read_to_string(e.path()).ok()?;
            if raw.contains("REFUSE_ALWAYS_PATH")
                && !e.path().ends_with("stage_executor_parity_policy.rs")
            {
                Some(e.path().display().to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for path in always_refuse_markers {
        offenders.push(format!(
            "REFUSE_ALWAYS marker requires issue id + deprecation/out_of_scope entry: {path}"
        ));
    }

    assert!(
        offenders.is_empty(),
        "stage executor parity policy violations:\n{}",
        offenders.join("\n")
    );
}
