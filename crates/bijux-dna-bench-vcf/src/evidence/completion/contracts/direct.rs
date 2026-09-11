use super::{model::PublishedReadinessReport, read_governed_json};
use crate::evidence::completion::ports::{DirectReadinessPorts, DirectReadinessStage};
use anyhow::{bail, Result};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
pub(crate) fn render_and_validate_direct_vcf_ready_output(
    repo_root: &Path,
    ports: &DirectReadinessPorts<'_>,
    stage: DirectReadinessStage,
    label: &str,
    expected_rows: u64,
) -> Result<()> {
    let stage_id = stage.stage_id();
    let output_path = (ports.output_path)(stage);
    let render_result = (ports.render)(repo_root, stage, PathBuf::from(output_path));
    let admission_blockers = (ports.external_bindings)(repo_root)?
        .into_iter()
        .filter(|(candidate, _)| candidate == stage_id)
        .map(|(_, tool)| tool)
        .collect::<BTreeSet<_>>();
    if !admission_blockers.is_empty() {
        let structural_status = render_result
            .as_ref()
            .map_or_else(|error| format!("incomplete: {error}"), |()| "complete".to_string());
        bail!(
            "blocked_external_access: {label} lacks checksum-admitted execution evidence for {}; interface_smoke={structural_status}",
            admission_blockers.into_iter().collect::<Vec<_>>().join(", ")
        );
    }
    render_result?;
    let payload: PublishedReadinessReport = read_governed_json(repo_root, output_path)?;
    if payload.retained_row_count != Some(expected_rows)
        || payload.active_row_count != Some(expected_rows)
        || payload.complete_row_count != Some(expected_rows)
        || payload.incomplete_row_count != Some(0)
        || payload.checked_surface_count != Some(8)
        || payload.violation_count != Some(0)
        || payload.ok != Some(true)
    {
        bail!(
            "{label} drifted from the governed complete pass state: retained={}, active={}, complete={}, incomplete={}, checked_surfaces={}, violations={}, ok={}",
            payload.retained_row_count.unwrap_or_default(),
            payload.active_row_count.unwrap_or_default(),
            payload.complete_row_count.unwrap_or_default(),
            payload.incomplete_row_count.unwrap_or_default(),
            payload.checked_surface_count.unwrap_or_default(),
            payload.violation_count.unwrap_or_default(),
            payload.ok.unwrap_or(false)
        );
    }
    Ok(())
}
