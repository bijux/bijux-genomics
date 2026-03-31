use std::collections::BTreeSet;

use super::manifest::parse_manifest;

#[must_use]
pub fn stage_input_ids(stage_id: &str) -> Option<BTreeSet<String>> {
    parse_manifest(stage_id).map(|manifest| {
        manifest
            .inputs
            .into_iter()
            .map(|port| port.name)
            .collect::<BTreeSet<_>>()
    })
}

#[must_use]
pub fn stage_output_ids(stage_id: &str) -> Option<BTreeSet<String>> {
    stage_output_ids_in_manifest_order(stage_id).map(|outputs| outputs.into_iter().collect())
}

#[must_use]
pub fn stage_output_ids_in_manifest_order(stage_id: &str) -> Option<Vec<String>> {
    parse_manifest(stage_id).map(|manifest| {
        manifest
            .outputs
            .into_iter()
            .map(|port| port.name)
            .collect::<Vec<_>>()
    })
}

#[must_use]
pub fn stage_parameter_ids(stage_id: &str) -> Option<BTreeSet<String>> {
    parse_manifest(stage_id).map(|manifest| {
        manifest
            .parameters
            .into_iter()
            .map(|parameter| parameter.name)
            .collect::<BTreeSet<_>>()
    })
}

#[must_use]
pub fn stage_compatible_tool_ids(stage_id: &str) -> Option<Vec<String>> {
    parse_manifest(stage_id).map(|manifest| manifest.compatible_tools)
}
