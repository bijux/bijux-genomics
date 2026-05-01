#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::path::PathBuf;

use cargo_metadata::MetadataCommand;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__crate_responsibility_matrix_policy__workspace_crates_are_listed_in_boundary_inventory(
) {
    let root = repo_root();
    let config_path = root.join("configs/ci/crate-boundaries.toml");
    let raw = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", config_path.display()));
    let value: toml::Value =
        toml::from_str(&raw).unwrap_or_else(|err| panic!("parse {}: {err}", config_path.display()));
    let rows = value.get("crates").and_then(toml::Value::as_array).cloned().unwrap_or_default();

    let listed = rows
        .iter()
        .filter_map(|row| row.get("name").and_then(toml::Value::as_str))
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();
    let workspace = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .no_deps()
        .exec()
        .unwrap_or_else(|err| panic!("cargo metadata: {err}"));
    let expected =
        workspace.packages.into_iter().map(|package| package.name).collect::<BTreeSet<_>>();

    bijux_dna_policies::policy_assert!(
        listed == expected,
        "configs/ci/crate-boundaries.toml must list every workspace crate exactly.\nlisted_only={:?}\nexpected_only={:?}",
        listed.difference(&expected).collect::<Vec<_>>(),
        expected.difference(&listed).collect::<Vec<_>>()
    );
}

#[test]
fn policy__contracts__crate_responsibility_matrix_policy__boundary_inventory_rows_are_complete() {
    let root = repo_root();
    let config_path = root.join("configs/ci/crate-boundaries.toml");
    let raw = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", config_path.display()));
    let value: toml::Value =
        toml::from_str(&raw).unwrap_or_else(|err| panic!("parse {}: {err}", config_path.display()));
    let rows = value.get("crates").and_then(toml::Value::as_array).cloned().unwrap_or_default();

    let mut offenders = Vec::new();
    for row in rows {
        let name = row.get("name").and_then(toml::Value::as_str).unwrap_or("<missing>");
        for key in ["family", "layer"] {
            if row.get(key).and_then(toml::Value::as_str).is_none() {
                offenders.push(format!("{name}: missing `{key}`"));
            }
        }
        let public_surfaces =
            row.get("public_surfaces").and_then(toml::Value::as_array).cloned().unwrap_or_default();
        if public_surfaces.is_empty() {
            offenders.push(format!("{name}: missing public_surfaces"));
            continue;
        }
        for surface in public_surfaces {
            let Some(rel) = surface.as_str() else {
                offenders.push(format!("{name}: non-string public surface entry"));
                continue;
            };
            let path = root.join(rel);
            if !path.exists() {
                offenders.push(format!("{name}: missing {}", path.display()));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "crate boundary inventory rows must stay complete:\n{}",
        offenders.join("\n")
    );
}
