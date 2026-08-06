//! Owner: bijux-dna-bench-model
//! Validation for the governed scientific data index consumed by benchmarks.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, FixedOffset};

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkDataIndex, BenchmarkDataIndexFile, BenchmarkDataIndexRecord, BenchmarkEligibility,
    DataIntegrityStatus,
};

use super::BENCHMARK_DATA_INDEX_SCHEMA_V1;

/// Validate persisted data-index identity, provenance, ordering, and aggregate closure.
///
/// # Errors
/// Returns an error when the index is malformed or its summary differs from its records.
pub fn validate_benchmark_data_index(index: &BenchmarkDataIndex) -> Result<(), BenchError> {
    if index.schema_version != BENCHMARK_DATA_INDEX_SCHEMA_V1 {
        return Err(invalid("benchmark data-index schema mismatch"));
    }
    validate_utc_timestamp(&index.created_at_utc)?;
    if index.data_root != "study://data" {
        return Err(invalid("benchmark data-index root is not canonical"));
    }
    if index.records.is_empty() {
        return Err(invalid("benchmark data index contains no records"));
    }

    let mut previous_id = None;
    let mut file_count = 0_u64;
    let mut size_bytes = 0_u64;
    let mut integrity_verified_count = 0_u64;
    let mut eligibility = BTreeMap::new();
    for record in &index.records {
        validate_record(record)?;
        if previous_id.is_some_and(|value: &str| value >= record.id.as_str()) {
            return Err(invalid("benchmark data-index records are not strictly ordered"));
        }
        previous_id = Some(record.id.as_str());
        file_count = file_count
            .checked_add(
                u64::try_from(record.files.len()).map_err(|_| invalid("file count overflows"))?,
            )
            .ok_or_else(|| invalid("file count overflows"))?;
        size_bytes = record.files.iter().try_fold(size_bytes, |total, file| {
            total.checked_add(file.size_bytes).ok_or_else(|| invalid("file size total overflows"))
        })?;
        if record.integrity_status == DataIntegrityStatus::Verified {
            integrity_verified_count = integrity_verified_count
                .checked_add(1)
                .ok_or_else(|| invalid("verified record count overflows"))?;
        }
        let count = eligibility.entry(record.benchmark_eligibility).or_insert(0_u64);
        *count = count.checked_add(1).ok_or_else(|| invalid("eligibility count overflows"))?;
    }

    let record_count =
        u64::try_from(index.records.len()).map_err(|_| invalid("record count overflows"))?;
    if index.summary.record_count != record_count
        || index.summary.file_count != file_count
        || index.summary.size_bytes != size_bytes
        || index.summary.integrity_verified_count != integrity_verified_count
        || index.summary.benchmark_eligibility != eligibility
    {
        return Err(invalid("benchmark data-index summary does not close over its records"));
    }
    Ok(())
}

fn validate_record(record: &BenchmarkDataIndexRecord) -> Result<(), BenchError> {
    validate_identifier(&record.id, "record")?;
    validate_study_uri(&record.catalog_path, "catalog")?;
    if !record.catalog_path.ends_with(&format!("/{}.json", record.id)) {
        return Err(invalid("benchmark data-index catalog path disagrees with record identity"));
    }
    validate_digest(&record.catalog_sha256, "catalog")?;
    for (value, label) in [
        (&record.purpose, "purpose"),
        (&record.access_classification, "access classification"),
        (&record.redistribution_classification, "redistribution classification"),
    ] {
        validate_nonempty(value, label)?;
    }
    if record.known_limitations.is_empty()
        || record.known_limitations.iter().any(|value| value.trim().is_empty())
    {
        return Err(invalid("benchmark data-index known limitations are incomplete"));
    }
    validate_temporal_provenance(record)?;

    let mut previous_path = None;
    let mut paths = BTreeSet::new();
    for file in &record.files {
        validate_file(file)?;
        if previous_path.is_some_and(|value: &str| value >= file.path.as_str()) {
            return Err(invalid("benchmark data-index files are not strictly ordered"));
        }
        previous_path = Some(file.path.as_str());
        if !paths.insert(file.path.as_str()) {
            return Err(invalid("benchmark data-index contains duplicate file paths"));
        }
    }

    let mut missing = Vec::new();
    if record.files.is_empty() {
        missing.push("governed_files");
    }
    if record.source_identities.is_empty() {
        missing.push("source_identity");
    }
    if record.temporal_provenance.timestamp.is_none() {
        missing.push("temporal_provenance");
    }
    if record.missing_provenance.iter().map(String::as_str).collect::<Vec<_>>() != missing {
        return Err(invalid("benchmark data-index missing provenance is inconsistent"));
    }
    let expected_integrity = if record.files.is_empty() {
        DataIntegrityStatus::Unverified
    } else {
        DataIntegrityStatus::Verified
    };
    let expected_eligibility = if missing.is_empty() {
        BenchmarkEligibility::Eligible
    } else {
        BenchmarkEligibility::Incomplete
    };
    if record.integrity_status != expected_integrity
        || record.benchmark_eligibility != expected_eligibility
    {
        return Err(invalid("benchmark data-index admission status is inconsistent"));
    }
    Ok(())
}

fn validate_file(file: &BenchmarkDataIndexFile) -> Result<(), BenchError> {
    validate_study_uri(&file.path, "file")?;
    validate_digest(&file.sha256, "file")?;
    validate_nonempty(&file.role, "file role")?;
    if file.legacy_fields.is_empty()
        || file.legacy_fields.iter().any(|value| value.trim().is_empty())
    {
        return Err(invalid("benchmark data-index legacy file identity is incomplete"));
    }
    Ok(())
}

fn validate_temporal_provenance(record: &BenchmarkDataIndexRecord) -> Result<(), BenchError> {
    match (
        record.temporal_provenance.field.as_deref(),
        record.temporal_provenance.timestamp.as_deref(),
    ) {
        (Some(field), Some(timestamp)) => {
            validate_nonempty(field, "temporal provenance field")?;
            validate_utc_timestamp(timestamp)
        }
        (None, None) => Ok(()),
        _ => Err(invalid("benchmark data-index temporal provenance is incomplete")),
    }
}

fn validate_utc_timestamp(value: &str) -> Result<(), BenchError> {
    let timestamp = DateTime::<FixedOffset>::parse_from_rfc3339(value)
        .map_err(|_| invalid("benchmark data-index timestamp is not RFC 3339"))?;
    if timestamp.offset().local_minus_utc() != 0 {
        return Err(invalid("benchmark data-index timestamp is not UTC"));
    }
    Ok(())
}

fn validate_study_uri(value: &str, label: &str) -> Result<(), BenchError> {
    let relative = value
        .strip_prefix("study://data/")
        .ok_or_else(|| invalid(&format!("benchmark data-index {label} path is not canonical")))?;
    if relative.is_empty()
        || relative.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(invalid(&format!("benchmark data-index {label} path is not normalized")));
    }
    Ok(())
}

fn validate_identifier(value: &str, label: &str) -> Result<(), BenchError> {
    let mut bytes = value.bytes();
    if !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        || !bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        return Err(invalid(&format!("benchmark data-index {label} identity is malformed")));
    }
    Ok(())
}

fn validate_digest(value: &str, label: &str) -> Result<(), BenchError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || value.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid(&format!("benchmark data-index {label} digest is malformed")));
    }
    Ok(())
}

fn validate_nonempty(value: &str, label: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() || value != value.trim() {
        return Err(invalid(&format!("benchmark data-index {label} is empty or malformed")));
    }
    Ok(())
}

fn invalid(message: &str) -> BenchError {
    BenchError::InvalidPolicy(message.to_string())
}
