#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

use toml::Value;

const FOUNDATION_CRATES: &[&str] = &[
    "bijux-dna",
    "bijux-dna-api",
    "bijux-dna-core",
    "bijux-dna-dev",
    "bijux-dna-engine",
    "bijux-dna-infra",
    "bijux-dna-policies",
    "bijux-dna-runner",
    "bijux-dna-runtime",
    "bijux-dna-testkit",
];

#[test]
fn policy__boundaries__foundation_lints__foundation_crates_apply_workspace_lints() {
    let workspace = workspace_root();

    for crate_name in FOUNDATION_CRATES {
        let manifest_path = workspace.join("crates").join(crate_name).join("Cargo.toml");
        let manifest = std::fs::read_to_string(&manifest_path)
            .unwrap_or_else(|err| panic!("read {}: {err}", manifest_path.display()));

        assert!(
            has_workspace_lints(&manifest),
            "{crate_name} must opt into the workspace lint table"
        );
    }
}

#[test]
fn policy__boundaries__foundation_lints__shared_test_crates_use_workspace_catalog() {
    let workspace = workspace_root();

    for crate_name in FOUNDATION_CRATES {
        let manifest_path = workspace.join("crates").join(crate_name).join("Cargo.toml");
        let manifest = std::fs::read_to_string(&manifest_path)
            .unwrap_or_else(|err| panic!("read {}: {err}", manifest_path.display()));

        for crate_dependency in ["bijux-dna-policies", "bijux-dna-testkit"] {
            let ad_hoc_path_prefix = format!("{crate_dependency} = {{ path = ");
            assert!(
                !manifest.contains(&ad_hoc_path_prefix),
                "{crate_name} must use {crate_dependency}.workspace = true instead of an ad hoc path dependency"
            );
        }
    }
}

#[test]
fn policy__boundaries__foundation_lints__manifest_sections_are_visually_separated() {
    let workspace = workspace_root();

    for crate_name in FOUNDATION_CRATES {
        let manifest_path = workspace.join("crates").join(crate_name).join("Cargo.toml");
        let manifest = std::fs::read_to_string(&manifest_path)
            .unwrap_or_else(|err| panic!("read {}: {err}", manifest_path.display()));

        let mut previous_line = "";
        for line in manifest.lines() {
            if line.starts_with('[') {
                assert!(
                    previous_line.trim().is_empty(),
                    "{crate_name} Cargo.toml section {line} must be preceded by a blank line"
                );
            }
            previous_line = line;
        }
    }
}

#[test]
fn policy__boundaries__foundation_lints__external_dependencies_use_workspace_catalog() {
    let workspace = workspace_root();

    for crate_name in FOUNDATION_CRATES {
        let manifest_path = workspace.join("crates").join(crate_name).join("Cargo.toml");
        let manifest = std::fs::read_to_string(&manifest_path)
            .unwrap_or_else(|err| panic!("read {}: {err}", manifest_path.display()));
        let manifest = manifest
            .parse::<Value>()
            .unwrap_or_else(|err| panic!("parse {}: {err}", manifest_path.display()));

        for section in ["dependencies", "dev-dependencies"] {
            let Some(dependencies) = manifest.get(section).and_then(Value::as_table) else {
                continue;
            };

            for (dependency_name, dependency_spec) in dependencies {
                assert!(
                    !uses_inline_registry_version(dependency_spec),
                    "{crate_name} {section}.{dependency_name} must use the workspace dependency catalog instead of an inline registry version"
                );
            }
        }
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("resolve workspace root"))
        .to_path_buf()
}

fn has_workspace_lints(manifest: &str) -> bool {
    let mut in_lints = false;

    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_lints = line == "[lints]";
            continue;
        }
        if in_lints && line == "workspace = true" {
            return true;
        }
    }

    false
}

fn uses_inline_registry_version(dependency_spec: &Value) -> bool {
    dependency_spec.as_str().is_some()
        || dependency_spec
            .as_table()
            .is_some_and(|table| table.contains_key("version") && !table.contains_key("workspace"))
}
