use crate::{model::RealOutputParserSmokeReport, values::repo_relative_path};
use anyhow::{anyhow, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// Publish a consistent successful report using atomic per-file replacement.
///
/// # Errors
/// Rejects failed or inconsistent evidence before replacing prior publication.
pub fn publish_report(
    repo_root: &Path,
    output_path: &Path,
    report: &RealOutputParserSmokeReport,
) -> Result<()> {
    let mut ids = BTreeSet::new();
    let mut counts = BTreeMap::new();
    for row in &report.rows {
        if !row.passed || !ids.insert(&row.family_id) {
            return Err(anyhow!("parser proof rows must be successful and unique"));
        }
        *counts.entry(row.parser_surface.clone()).or_insert(0usize) += 1;
    }
    if report.family_count == 0
        || report.family_count != report.rows.len()
        || report.passed_family_count != report.rows.len()
        || report.failed_family_count != 0
        || report.parser_surface_counts != counts
    {
        return Err(anyhow!("parser proof summary counts disagree with admitted rows"));
    }
    bijux_dna_infra::atomic_write_json(&repo_relative_path(repo_root, output_path), report)?;
    Ok(())
}
