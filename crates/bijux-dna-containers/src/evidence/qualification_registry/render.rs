//! Owner: bijux-dna-containers
//! Qualification registry stable registry and run-index serialization.

use super::admission::validate_container_run_index;
use super::{ContainerRunIndex, QualificationRunRegistry, QualificationRunState};
use anyhow::{Context, Result};

/// Render deterministic checked-in TOML.
///
/// # Errors
/// Returns an error when registry serialization fails.
pub fn render_qualification_run_registry(registry: &QualificationRunRegistry) -> Result<String> {
    Ok(format!(
        "# GENERATED FILE - DO NOT EDIT\n{}",
        toml::to_string_pretty(registry).context("serialize qualification-run registry")?
    ))
}

/// Render deterministic container run-index JSON.
///
/// # Errors
/// Returns an error when serialization fails.
pub fn render_container_run_index_json(index: &ContainerRunIndex) -> Result<String> {
    validate_container_run_index(index)?;
    Ok(serde_json::to_string_pretty(index)? + "\n")
}

#[must_use]
pub fn render_container_run_index_markdown(index: &ContainerRunIndex) -> String {
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-containers::evidence::qualification_registry -->".to_string(),
        "# Container run index".to_string(),
        String::new(),
        "Run IDs are permanent protocols; scheduler jobs and retries are attempts under those IDs."
            .to_string(),
        String::new(),
        format!("- Registered protocols: **{}**", index.summary.run_count),
        format!("- Active protocols: **{}**", index.summary.active_count),
        format!("- Allocated protocols: **{}**", index.summary.allocated_count),
        format!("- Historical protocols: **{}**", index.summary.historical_count),
        format!("- Reserved IDs: **{}**", index.summary.reserved_count),
        String::new(),
        "| Run | State | Stage | Tools | Citation | Evidence |".to_string(),
        "|---|---|---|---|---|---|".to_string(),
    ];
    for run in &index.runs {
        let state = match run.state {
            QualificationRunState::Campaign => "campaign",
            QualificationRunState::Active => "active",
            QualificationRunState::Allocated => "allocated",
            QualificationRunState::Historical => "historical",
            QualificationRunState::Reserved => "reserved",
        };
        let evidence =
            if run.citation.evidence_uris.is_empty() { "not selected" } else { "indexed" };
        lines.push(format!(
            "| `{}` | `{state}` | `{}` | {} | `{}` | {evidence} |",
            run.run_id,
            run.stage_id,
            run.tool_ids.iter().map(|tool| format!("`{tool}`")).collect::<Vec<_>>().join(", "),
            run.citation.citation_key,
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}
