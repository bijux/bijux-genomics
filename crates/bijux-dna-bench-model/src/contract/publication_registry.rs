//! Owner: bijux-dna-bench-model
//! Validation for persisted benchmark publication authority.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::Path;

use crate::model::{BenchmarkPublicationRegistry, BenchmarkStagePublicationDisposition};
use crate::BenchError;

use super::BENCHMARK_PUBLICATION_REGISTRY_SCHEMA_V1;

/// Validate publication counts, identities, paths, digests, and quarantine policy.
///
/// # Errors
/// Returns [`BenchError`] when any persisted registry invariant is incomplete or contradictory.
pub fn validate_benchmark_publication_registry(
    registry: &BenchmarkPublicationRegistry,
) -> Result<(), BenchError> {
    if registry.schema_version != BENCHMARK_PUBLICATION_REGISTRY_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy("unsupported publication registry schema".into()));
    }
    if registry.summary.current_product_count != registry.current_products.len() {
        return Err(BenchError::InvalidPolicy("current publication count drifted".into()));
    }
    let active = registry
        .stage_reports
        .iter()
        .filter(|row| row.disposition == BenchmarkStagePublicationDisposition::Active)
        .count();
    let historical = registry.stage_reports.len() - active;
    if registry.summary.active_stage_report_count != active
        || registry.summary.historical_stage_report_count != historical
    {
        return Err(BenchError::InvalidPolicy("stage publication counts drifted".into()));
    }
    if registry.summary.quarantined_surface_count != registry.quarantined_surfaces.len()
        || registry.summary.quarantined_file_count
            != registry
                .quarantined_surfaces
                .iter()
                .map(|surface| surface.tracked_file_count)
                .sum::<usize>()
    {
        return Err(BenchError::InvalidPolicy("quarantined publication counts drifted".into()));
    }

    let mut product_ids = BTreeSet::new();
    let mut publication_paths = BTreeSet::new();
    for product in &registry.current_products {
        if product.product_id.is_empty() || !product_ids.insert(product.product_id.as_str()) {
            return Err(BenchError::InvalidPolicy("duplicate or empty publication product".into()));
        }
        validate_publication_pair(
            &product.json_path,
            &product.json_sha256,
            &product.markdown_path,
            &product.markdown_sha256,
            &mut publication_paths,
        )?;
    }
    let mut stage_identities = BTreeSet::new();
    for report in &registry.stage_reports {
        if !is_run_id(&report.run_id) || report.stage_id.is_empty() {
            return Err(BenchError::InvalidPolicy("invalid stage publication identity".into()));
        }
        validate_hex(&report.source_commit, 40, "stage publication source commit")?;
        if !stage_identities.insert((
            report.run_id.as_str(),
            report.source_commit.as_str(),
            report.json_path.as_str(),
        )) {
            return Err(BenchError::InvalidPolicy("duplicate stage publication identity".into()));
        }
        validate_publication_pair(
            &report.json_path,
            &report.json_sha256,
            &report.markdown_path,
            &report.markdown_sha256,
            &mut publication_paths,
        )?;
    }
    let mut quarantine_paths = BTreeSet::new();
    for surface in &registry.quarantined_surfaces {
        if surface.path_prefix.is_empty()
            || surface.disposition != "quarantined_legacy_readiness"
            || surface.tracked_file_count == 0
            || surface.authority_policy.is_empty()
            || !quarantine_paths.insert(surface.path_prefix.as_str())
        {
            return Err(BenchError::InvalidPolicy(
                "invalid quarantined publication surface".into(),
            ));
        }
        validate_hex(&surface.tree_sha256, 64, "quarantined publication tree digest")?;
    }
    Ok(())
}

fn validate_publication_pair<'a>(
    json_path: &'a str,
    json_sha256: &str,
    markdown_path: &'a str,
    markdown_sha256: &str,
    paths: &mut BTreeSet<&'a str>,
) -> Result<(), BenchError> {
    if Path::new(json_path).extension() != Some(OsStr::new("json"))
        || Path::new(markdown_path).extension() != Some(OsStr::new("md"))
        || !paths.insert(json_path)
        || !paths.insert(markdown_path)
    {
        return Err(BenchError::InvalidPolicy("invalid or duplicate publication path".into()));
    }
    validate_hex(json_sha256, 64, "publication JSON digest")?;
    validate_hex(markdown_sha256, 64, "publication Markdown digest")
}

fn validate_hex(value: &str, length: usize, label: &str) -> Result<(), BenchError> {
    if value.len() != length || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(BenchError::InvalidPolicy(format!("invalid {label}")));
    }
    Ok(())
}

fn is_run_id(value: &str) -> bool {
    value.len() == 8
        && value.starts_with("run-")
        && value[4..].bytes().all(|byte| byte.is_ascii_digit())
}
