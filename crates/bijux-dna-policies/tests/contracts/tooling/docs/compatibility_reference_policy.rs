#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn policy__contracts__compatibility_reference_policy__compatibility_sources_and_indexes_are_present() {
    let root = workspace_root();
    let ci_index = std::fs::read_to_string(root.join("configs/ci/index.md")).expect("read configs/ci/index.md");
    let compat_index =
        std::fs::read_to_string(root.join("configs/ci/compatibility/index.md")).expect("read configs/ci/compatibility/index.md");
    let reference_index =
        std::fs::read_to_string(root.join("docs/50-reference/index.md")).expect("read docs/50-reference/index.md");

    bijux_dna_policies::policy_assert!(
        ci_index.contains("configs/ci/compatibility/"),
        "configs/ci/index.md must reference configs/ci/compatibility/"
    );
    for needle in ["deprecations.toml", "release_changes.toml"] {
        bijux_dna_policies::policy_assert!(
            compat_index.contains(needle),
            "configs/ci/compatibility/index.md missing {needle}"
        );
    }
    for needle in [
        "SCHEMA_REGISTRY.md",
        "API_VERSIONING.md",
        "DEPRECATION_DASHBOARD.md",
        "UPGRADE_GUIDE.md",
    ] {
        bijux_dna_policies::policy_assert!(
            reference_index.contains(needle),
            "docs/50-reference/index.md missing {needle}"
        );
    }
}

#[test]
fn policy__contracts__compatibility_reference_policy__generated_docs_and_normative_links_stay_intact() {
    let root = workspace_root();
    let schema_registry =
        std::fs::read_to_string(root.join("docs/50-reference/SCHEMA_REGISTRY.md")).expect("read schema registry");
    let api_versioning =
        std::fs::read_to_string(root.join("docs/50-reference/API_VERSIONING.md")).expect("read api versioning");
    let dashboard =
        std::fs::read_to_string(root.join("docs/50-reference/DEPRECATION_DASHBOARD.md")).expect("read deprecation dashboard");
    let upgrade_guide =
        std::fs::read_to_string(root.join("docs/50-reference/UPGRADE_GUIDE.md")).expect("read upgrade guide");
    let contract_compatibility =
        std::fs::read_to_string(root.join("docs/50-reference/CONTRACT_COMPATIBILITY.md")).expect("read contract compatibility");
    let manifest_migration =
        std::fs::read_to_string(root.join("docs/50-reference/MANIFEST_MIGRATION.md")).expect("read manifest migration");

    for doc in [&schema_registry, &api_versioning, &dashboard, &upgrade_guide] {
        bijux_dna_policies::policy_assert!(
            doc.contains("<!-- GENERATED FILE - DO NOT EDIT -->"),
            "compatibility reference docs must remain generated outputs"
        );
    }
    for needle in ["Schema Families", "Durable Error Codes"] {
        bijux_dna_policies::policy_assert!(
            schema_registry.contains(needle),
            "SCHEMA_REGISTRY.md missing required section {needle}"
        );
    }
    bijux_dna_policies::policy_assert!(
        api_versioning.contains("v1.execute") && api_versioning.contains("ExecuteResponse"),
        "API_VERSIONING.md must document the governed v1 route inventory"
    );
    for needle in ["stage_id", "tool_id", "metric_id", "param", "field"] {
        bijux_dna_policies::policy_assert!(
            dashboard.contains(needle),
            "DEPRECATION_DASHBOARD.md missing category {needle}"
        );
    }
    for needle in ["Schemas", "Defaults", "Tools", "Containers", "Evidence Expectations"] {
        bijux_dna_policies::policy_assert!(
            upgrade_guide.contains(needle),
            "UPGRADE_GUIDE.md missing area section {needle}"
        );
    }
    bijux_dna_policies::policy_assert!(
        contract_compatibility.contains("SCHEMA_REGISTRY.md")
            && contract_compatibility.contains("API_VERSIONING.md"),
        "CONTRACT_COMPATIBILITY.md must point to generated schema and API compatibility references"
    );
    bijux_dna_policies::policy_assert!(
        manifest_migration.contains("UPGRADE_GUIDE.md")
            && manifest_migration.contains("SCHEMA_REGISTRY.md"),
        "MANIFEST_MIGRATION.md must point to generated upgrade and schema references"
    );
}
