#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn scientific_owned_fields(root: &std::path::Path) -> Vec<String> {
    let raw = std::fs::read_to_string(root.join("configs/ci/policy_layers.toml"))
        .unwrap_or_else(|err| panic!("read policy_layers.toml: {err}"));
    let value: toml::Value = toml::from_str(&raw).unwrap_or_else(|err| panic!("parse policy_layers.toml: {err}"));
    value["scientific"]["owned_fields"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(ToOwned::to_owned))
        .collect()
}

fn validate_operational_policy_override(raw: &str, forbidden_keys: &[String]) -> Vec<String> {
    forbidden_keys
        .iter()
        .filter(|key| raw.contains(key.as_str()))
        .map(|key| format!("operational config must not declare scientific field `{key}`"))
        .collect()
}

#[test]
fn policy__contracts__policy_layer_split_policy__operational_configs_do_not_declare_scientific_fields(
) {
    let root = repo_root();
    let forbidden_keys = scientific_owned_fields(&root);
    let mut offenders = Vec::new();

    for rel in ["configs/runtime", "configs/logging", "configs/hpc"] {
        for entry in WalkDir::new(root.join(rel))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("toml"))
        {
            let raw = std::fs::read_to_string(entry.path())
                .unwrap_or_else(|err| panic!("read {}: {err}", entry.path().display()));
            for violation in validate_operational_policy_override(&raw, &forbidden_keys) {
                offenders.push(format!("{}: {violation}", entry.path().display()));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "operational configs must not declare scientific policy fields:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__policy_layer_split_policy__negative_operational_override_fixtures_are_rejected(
) {
    let root = repo_root();
    let forbidden_keys = scientific_owned_fields(&root);

    let fixture_a = r#"
run_base_dir = "artifacts/runs"

[scientific_thresholds]
contamination_max = 0.05
"#;
    let fixture_b = r#"
executor = "local"

[sample_assumptions]
ancient_dna = true
"#;

    let violations_a = validate_operational_policy_override(fixture_a, &forbidden_keys);
    let violations_b = validate_operational_policy_override(fixture_b, &forbidden_keys);

    bijux_dna_policies::policy_assert!(
        !violations_a.is_empty() && !violations_b.is_empty(),
        "negative operational override fixtures must be rejected by the policy-layer validator"
    );
}
