#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;

#[test]
fn policy__contracts__image_catalog_completeness_policy__production_tools_have_image_entries() {
    let root = support::workspace_root();
    let registry_raw = std::fs::read_to_string(root.join("configs/ci/registry/tool_registry.toml"))
        .expect("read configs/ci/registry/tool_registry.toml");
    let images_raw = std::fs::read_to_string(root.join("configs/ci/tools/images.toml"))
        .expect("read configs/ci/tools/images.toml");

    let registry: toml::Value = registry_raw.parse().expect("parse tool_registry.toml");
    let images: toml::Value = images_raw.parse().expect("parse images.toml");

    let production_tools = registry
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|tool| {
            tool.get("status")
                .and_then(toml::Value::as_str)
                .is_some_and(support::registry_status_is_production)
        })
        .filter_map(|tool| tool.get("id").and_then(toml::Value::as_str).map(str::to_string))
        .collect::<BTreeSet<_>>();

    let image_tools = images
        .as_table()
        .map(|table| table.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();

    let mut offenders = Vec::new();
    for tool in production_tools {
        if !image_tools.contains(&tool) {
            offenders.push(tool);
        }
    }

    assert!(offenders.is_empty(), "production tools missing from image catalog: {offenders:?}");
}
