#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;

fn table_array<'a>(root: &'a toml::Value, key: &str) -> Vec<&'a toml::Value> {
    root.get(key)
        .and_then(toml::Value::as_array)
        .map(|rows| rows.iter().collect())
        .unwrap_or_default()
}

fn string_set(rows: &[&toml::Value], key: &str) -> BTreeSet<String> {
    rows.iter()
        .filter_map(|row| row.get(key).and_then(toml::Value::as_str))
        .map(str::to_string)
        .collect()
}

#[test]
fn policy__contracts__fastq_route_policy__route_registry_covers_required_families_and_graphs() {
    let root = support::workspace_root();
    let raw = std::fs::read_to_string(root.join("domain/fastq/route_policies.toml"))
        .expect("read domain/fastq/route_policies.toml");
    let parsed = raw.parse::<toml::Value>().expect("parse route_policies.toml");

    assert_eq!(
        parsed.get("schema_version").and_then(toml::Value::as_str),
        Some("bijux.fastq.route_policies.v1")
    );

    let route_families = table_array(&parsed, "route_family");
    let route_graphs = table_array(&parsed, "route_graph");
    assert_eq!(
        string_set(&route_families, "id"),
        [
            "amplicon_standard",
            "amplicon_umi",
            "host_associated_metagenome",
            "premerged_single_end",
            "shotgun_adna",
            "shotgun_standard",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>(),
        "FASTQ route policy must expose every required route family"
    );
    assert_eq!(
        string_set(&route_graphs, "id"),
        ["benchmark_cohort", "default_qc", "depletion_first", "minimal_triage"]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        "FASTQ route policy must expose required route graph templates"
    );

    let mut offenders = Vec::new();
    for row in route_families.iter().chain(route_graphs.iter()) {
        let id = row.get("id").and_then(toml::Value::as_str).unwrap_or("<missing>");
        if row
            .get("plan_certification")
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .is_empty()
        {
            offenders.push(format!("{id} missing plan_certification"));
        }
        if row.get("required_assets").and_then(toml::Value::as_array).is_none() {
            offenders.push(format!("{id} missing required_assets"));
        }
    }
    assert!(offenders.is_empty(), "route policy violations:\n{}", offenders.join("\n"));
}
