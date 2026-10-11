//! Owner: bijux-dna-bench-model
//! Admission rules for governed benchmark publication outcomes.

use std::collections::BTreeSet;

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkCampaignReportPublication, GovernedDataCatalogRegistryPublication,
    GovernedDataCatalogSnapshotPublication,
};
use crate::{
    validate_governed_data_catalog_registry, BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_REGISTRY_PUBLICATION_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1,
};

/// Validate one immutable governed-catalog snapshot publication outcome.
///
/// # Errors
/// Returns an error for schema, path, count, or embedded registry drift.
pub fn validate_governed_data_catalog_snapshot_publication(
    publication: &GovernedDataCatalogSnapshotPublication,
) -> Result<(), BenchError> {
    if publication.schema_version != GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1 {
        return Err(invalid("catalog snapshot publication schema mismatch"));
    }
    if !publication.candidate_registry_path.is_absolute()
        || publication.candidate_registry_path.file_name().and_then(|value| value.to_str())
            != Some("registry.json")
    {
        return Err(invalid("catalog snapshot publication path is invalid"));
    }
    let total = publication
        .preserved_snapshot_count
        .checked_add(publication.retained_snapshot_count)
        .ok_or_else(|| invalid("catalog snapshot publication count overflow"))?;
    if usize::try_from(total).ok() != Some(publication.registry.records.len()) {
        return Err(invalid("catalog snapshot publication counts drifted"));
    }
    validate_governed_data_catalog_registry(&publication.registry)
}

/// Validate one immutable governed-catalog registry publication outcome.
///
/// # Errors
/// Returns an error for schema, path, digest, or embedded registry drift.
pub fn validate_governed_data_catalog_registry_publication(
    publication: &GovernedDataCatalogRegistryPublication,
) -> Result<(), BenchError> {
    if publication.schema_version != GOVERNED_DATA_CATALOG_REGISTRY_PUBLICATION_SCHEMA_V1 {
        return Err(invalid("catalog registry publication schema mismatch"));
    }
    if !publication.registry_path.is_absolute()
        || publication.registry_path.extension().and_then(|value| value.to_str()) != Some("json")
        || !valid_sha256(&publication.registry_sha256)
    {
        return Err(invalid("catalog registry publication identity is invalid"));
    }
    validate_governed_data_catalog_registry(&publication.registry)
}

/// Validate one immutable benchmark campaign report publication outcome.
///
/// # Errors
/// Returns an error for schema, run/stage, path, digest, presentation-source, or overlay drift.
pub fn validate_benchmark_campaign_report_publication(
    publication: &BenchmarkCampaignReportPublication,
) -> Result<(), BenchError> {
    if publication.schema_version != BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1 {
        return Err(invalid("campaign report publication schema mismatch"));
    }
    if publication.run_id.trim().is_empty()
        || publication.stage_id.trim().is_empty()
        || !publication.stage_id.contains('.')
    {
        return Err(invalid("campaign report publication run or stage identity is invalid"));
    }
    if !publication.report_path.is_absolute()
        || !publication.markdown_path.is_absolute()
        || publication.report_path == publication.markdown_path
        || publication.report_path.extension().and_then(|value| value.to_str()) != Some("json")
        || publication.markdown_path.extension().and_then(|value| value.to_str()) != Some("md")
    {
        return Err(invalid("campaign report publication path identity is invalid"));
    }
    if !valid_sha256(&publication.report_sha256)
        || !valid_sha256(&publication.markdown_sha256)
        || !valid_sha256(&publication.presentation_catalog_sha256)
        || !valid_commit(&publication.presentation_repository_commit)
    {
        return Err(invalid("campaign report publication source identity is invalid"));
    }
    let mut overlays = BTreeSet::new();
    if publication
        .presentation_overlaid_stage_ids
        .iter()
        .any(|stage_id| stage_id.trim().is_empty() || !overlays.insert(stage_id.as_str()))
    {
        return Err(invalid("campaign report publication overlays are invalid"));
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        && value.bytes().any(|byte| byte != b'0')
}

fn valid_commit(value: &str) -> bool {
    value.len() == 40
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
