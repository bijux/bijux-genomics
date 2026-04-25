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

fn array_strings(row: &toml::Value, key: &str) -> Vec<String> {
    row.get(key)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect()
}

#[test]
fn policy__contracts__fastq_asset_hydration_policy__bundles_resolve_locks_and_routes() {
    let root = support::workspace_root();
    let locks = std::fs::read_to_string(root.join("configs/runtime/asset_locks.toml"))
        .expect("read asset_locks.toml")
        .parse::<toml::Value>()
        .expect("parse asset_locks.toml");
    let hydration = std::fs::read_to_string(root.join("configs/runtime/asset_hydration.toml"))
        .expect("read asset_hydration.toml")
        .parse::<toml::Value>()
        .expect("parse asset_hydration.toml");
    let routes = std::fs::read_to_string(root.join("domain/fastq/route_policies.toml"))
        .expect("read route_policies.toml")
        .parse::<toml::Value>()
        .expect("parse route_policies.toml");

    let lock_families = table_array(&locks, "lock_family")
        .into_iter()
        .filter_map(|row| row.get("id").and_then(toml::Value::as_str))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let route_ids = table_array(&routes, "route_family")
        .into_iter()
        .chain(table_array(&routes, "route_graph"))
        .filter_map(|row| row.get("id").and_then(toml::Value::as_str))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let bundle_rows = table_array(&hydration, "asset_bundle");
    let bundle_ids = bundle_rows
        .iter()
        .filter_map(|row| row.get("id").and_then(toml::Value::as_str))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();

    let mut offenders = Vec::new();
    for row in &bundle_rows {
        let id = row.get("id").and_then(toml::Value::as_str).unwrap_or("<missing>");
        let lock_family = row.get("lock_family").and_then(toml::Value::as_str).unwrap_or_default();
        if !lock_families.contains(lock_family) {
            offenders.push(format!("bundle {id} references unknown lock family {lock_family}"));
        }
        for route in array_strings(row, "required_for_routes") {
            if !route_ids.contains(&route) {
                offenders.push(format!("bundle {id} references unknown route {route}"));
            }
        }
        for key in ["verification_command", "hydration_command", "offline_replay_source"] {
            if row.get(key).and_then(toml::Value::as_str).unwrap_or_default().is_empty() {
                offenders.push(format!("bundle {id} missing {key}"));
            }
        }
    }

    let pseudo_assets = ["asset_locks", "input_hash_manifest"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let satisfiable_assets = lock_families.union(&bundle_ids).cloned().collect::<BTreeSet<_>>();
    for row in
        table_array(&routes, "route_family").into_iter().chain(table_array(&routes, "route_graph"))
    {
        let route_id = row.get("id").and_then(toml::Value::as_str).unwrap_or("<missing>");
        for asset in array_strings(row, "required_assets") {
            if !satisfiable_assets.contains(&asset) && !pseudo_assets.contains(&asset) {
                offenders.push(format!("route {route_id} requires unsatisfied asset {asset}"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "fastq asset hydration policy violations:\n{}",
        offenders.join("\n")
    );
}
