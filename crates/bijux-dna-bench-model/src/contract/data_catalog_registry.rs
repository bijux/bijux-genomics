//! Owner: bijux-dna-bench-model
//! Validation for immutable governed data catalog selection.

use std::collections::BTreeSet;

use crate::diagnostics::BenchError;
use crate::model::{GovernedDataCatalogRegistry, GovernedDataCatalogSnapshot};

use super::{
    validate_utc_timestamp, GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V2,
};

/// Validate registry ordering, immutable snapshot identity, and current-v2 selection.
///
/// # Errors
/// Returns an error when any registry identity is ambiguous, mutable, or malformed.
pub fn validate_governed_data_catalog_registry(
    registry: &GovernedDataCatalogRegistry,
) -> Result<(), BenchError> {
    if registry.schema_version != GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1 {
        return Err(invalid("governed data catalog registry schema mismatch"));
    }
    validate_utc_timestamp(&registry.created_at_utc, "catalog registry creation")?;
    if registry.data_root != "study://data" {
        return Err(invalid("governed data catalog registry root is not canonical"));
    }
    if registry.records.is_empty() {
        return Err(invalid("governed data catalog registry contains no records"));
    }

    let mut previous_id = None;
    for record in &registry.records {
        validate_identifier(&record.id)?;
        if previous_id.is_some_and(|value: &str| value >= record.id.as_str()) {
            return Err(invalid("governed data catalog registry records are not strictly ordered"));
        }
        previous_id = Some(record.id.as_str());
        validate_sha256(&record.current_catalog_sha256, "current catalog")?;
        if record.snapshots.is_empty() {
            return Err(invalid("governed data catalog registry record has no snapshots"));
        }

        let mut previous_path = None;
        let mut digests = BTreeSet::new();
        let mut current_schema = None;
        for snapshot in &record.snapshots {
            validate_snapshot(&record.id, snapshot)?;
            if previous_path.is_some_and(|value: &str| value >= snapshot.catalog_path.as_str()) {
                return Err(invalid("governed data catalog snapshots are not strictly ordered"));
            }
            previous_path = Some(snapshot.catalog_path.as_str());
            if !digests.insert(snapshot.catalog_sha256.as_str()) {
                return Err(invalid("governed data catalog registry repeats a snapshot digest"));
            }
            if snapshot.catalog_sha256 == record.current_catalog_sha256 {
                current_schema = Some(snapshot.catalog_schema_version.as_str());
            }
        }
        match current_schema {
            Some(GOVERNED_DATA_CATALOG_SCHEMA_V2) => {}
            Some(_) => {
                return Err(invalid("governed data catalog current snapshot is not schema v2"));
            }
            None => {
                return Err(invalid("governed data catalog current snapshot is not registered"));
            }
        }
    }
    Ok(())
}

fn validate_snapshot(id: &str, snapshot: &GovernedDataCatalogSnapshot) -> Result<(), BenchError> {
    validate_sha256(&snapshot.catalog_sha256, "catalog snapshot")?;
    if !matches!(
        snapshot.catalog_schema_version.as_str(),
        GOVERNED_DATA_CATALOG_SCHEMA_V1 | GOVERNED_DATA_CATALOG_SCHEMA_V2
    ) {
        return Err(invalid("governed data catalog snapshot schema is unsupported"));
    }
    let expected = format!("study://data/catalog/snapshots/{}/{id}.json", snapshot.source_commit);
    if snapshot.catalog_path != expected {
        return Err(invalid("governed data catalog snapshot path is not source addressed"));
    }
    validate_hex(&snapshot.source_commit, 40, "catalog snapshot source commit")?;
    validate_utc_timestamp(&snapshot.created_at_utc, "catalog snapshot creation")
}

fn validate_identifier(value: &str) -> Result<(), BenchError> {
    let mut bytes = value.bytes();
    if !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        || !bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        return Err(invalid("governed data catalog registry identity is malformed"));
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<(), BenchError> {
    validate_hex(value, 64, label)?;
    if value.bytes().all(|byte| byte == b'0') {
        return Err(invalid(&format!("{label} is all zeroes")));
    }
    Ok(())
}

fn validate_hex(value: &str, length: usize, label: &str) -> Result<(), BenchError> {
    if value.len() != length
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid(&format!("{label} is malformed")));
    }
    Ok(())
}

fn invalid(message: &str) -> BenchError {
    BenchError::InvalidPolicy(message.to_string())
}
