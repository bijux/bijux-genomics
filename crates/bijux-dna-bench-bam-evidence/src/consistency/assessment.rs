use super::models::BamActiveRowConsistencySurfaceCheck;
use crate::observations::{BindingKey, BindingSurface};
use std::collections::BTreeSet;
pub(super) fn build_surface_check(
    surface_id: &str,
    surface: &BindingSurface,
    active_bindings: &BTreeSet<BindingKey>,
    success_detail: &str,
) -> BamActiveRowConsistencySurfaceCheck {
    let output_path = &surface.output_path;
    let row_count = surface.row_count;
    let stage_count = surface.stage_count;
    let tool_count = surface.tool_count;
    let surface_bindings = surface.bindings.iter().cloned().collect::<BTreeSet<_>>();

    let binding_count = surface_bindings.len();
    let missing_bindings = diff_bindings(active_bindings, &surface_bindings);
    let extra_bindings = diff_bindings(&surface_bindings, active_bindings);
    let ok = row_count == active_bindings.len()
        && binding_count == active_bindings.len()
        && missing_bindings.is_empty()
        && extra_bindings.is_empty();
    let detail = if ok {
        success_detail.to_string()
    } else {
        format!(
            "surface `{surface_id}` drifted from active BAM rows: rows={row_count}, bindings={binding_count}, missing={}, extra={}",
            join_or_none(&missing_bindings),
            join_or_none(&extra_bindings),
        )
    };

    BamActiveRowConsistencySurfaceCheck {
        surface_id: surface_id.to_string(),
        output_path: output_path.clone(),
        row_count,
        stage_count,
        tool_count,
        binding_count,
        ok,
        missing_bindings,
        extra_bindings,
        detail,
    }
}
fn diff_bindings(left: &BTreeSet<BindingKey>, right: &BTreeSet<BindingKey>) -> Vec<String> {
    left.difference(right)
        .map(|binding| format!("{}/{}", binding.stage_id, binding.tool_id))
        .collect::<Vec<_>>()
}
fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
