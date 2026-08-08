//! Owner: bijux-dna-bench
//! Derivation and publication of the governed scientific data index.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_data_index, validate_governed_data_catalog_registry, BenchmarkDataIndex,
    BenchmarkDataIndexFile, BenchmarkDataIndexRecord, BenchmarkDataIndexSummary,
    BenchmarkEligibility, BenchmarkTemporalProvenance, DataIntegrityStatus,
    GovernedDataCatalogRegistry, GovernedDataCatalogSnapshot, BENCHMARK_DATA_INDEX_SCHEMA_V2,
    GOVERNED_DATA_CATALOG_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V2,
};
use chrono::{DateTime, FixedOffset};
use serde_json::{Map, Value};
use walkdir::WalkDir;

const TEMPORAL_FIELDS: [&str; 4] = [
    "retrieved_at_utc",
    "admitted_at_utc",
    "created_at_utc",
    "retrieved_from_existing_lunarc_holdings_at_utc",
];
const PATH_CONTRACTS: [(&str, &str, &str); 3] = [
    ("path", "sha256", "size_bytes"),
    ("governed_path", "governed_sha256", "governed_size_bytes"),
    ("derived_path", "derived_sha256", "derived_size_bytes"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkDataIndexRequest {
    pub catalog_registry_path: PathBuf,
    pub data_root: PathBuf,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkDataIndexPublicationRequest {
    pub index: BenchmarkDataIndexRequest,
    pub index_path: PathBuf,
    pub report_path: PathBuf,
}

/// Build a complete, checksum-verified data index from governed catalog records.
///
/// # Errors
/// Returns an error for malformed catalogs, ambiguous identities, unsafe paths, symlinks, missing
/// files, or size and digest drift.
pub fn build_benchmark_data_index(
    request: &BenchmarkDataIndexRequest,
) -> Result<BenchmarkDataIndex> {
    validate_utc_timestamp(&request.created_at_utc, "index creation")?;
    let data_root = canonical_directory(&request.data_root, "data root")?;
    let registry_path =
        governed_regular_file(&request.catalog_registry_path, &data_root, "catalog registry")?;
    let registry_bytes = fs::read(&registry_path)
        .with_context(|| format!("read catalog registry {}", registry_path.display()))?;
    let registry: GovernedDataCatalogRegistry = serde_json::from_slice(&registry_bytes)
        .with_context(|| format!("parse catalog registry {}", registry_path.display()))?;
    validate_governed_data_catalog_registry(&registry)
        .context("validate governed data catalog registry")?;
    let files_by_name = basename_index(&data_root)?;
    let mut records = registry
        .records
        .iter()
        .map(|record| {
            let snapshot = record
                .snapshots
                .iter()
                .find(|snapshot| snapshot.catalog_sha256 == record.current_catalog_sha256)
                .context("validated current catalog snapshot is absent")?;
            let path = resolve_study_data_uri(&snapshot.catalog_path, &data_root)?;
            normalize_record(&path, &data_root, &files_by_name, &record.id, snapshot)
        })
        .collect::<Result<Vec<_>>>()?;
    records.sort_by(|left, right| left.id.cmp(&right.id));
    let mut ids = BTreeSet::new();
    if records.iter().any(|record| !ids.insert(record.id.as_str())) {
        bail!("data catalog IDs are not unique");
    }

    let summary = summarize_records(&records)?;
    let index = BenchmarkDataIndex {
        schema_version: BENCHMARK_DATA_INDEX_SCHEMA_V2.to_string(),
        created_at_utc: request.created_at_utc.clone(),
        data_root: "study://data".to_string(),
        catalog_registry_path: Some(stable_data_path(&registry_path, &data_root)?),
        catalog_registry_sha256: Some(bijux_dna_infra::hash_bytes_sha256(&registry_bytes)),
        summary,
        records,
    };
    validate_benchmark_data_index(&index).context("validate derived benchmark data index")?;
    Ok(index)
}

/// Publish an immutable JSON index and human-readable report.
///
/// # Errors
/// Returns an error when derivation fails, a destination exists, or atomic publication fails.
pub fn publish_benchmark_data_index(
    request: &BenchmarkDataIndexPublicationRequest,
) -> Result<BenchmarkDataIndex> {
    for path in [&request.index_path, &request.report_path] {
        if path.exists() || path.is_symlink() {
            bail!("refusing to replace governed data-index publication: {}", path.display());
        }
    }
    let index = build_benchmark_data_index(&request.index)?;
    bijux_dna_infra::atomic_create_json(&request.index_path, &index)
        .with_context(|| format!("publish data index {}", request.index_path.display()))?;
    let report = render_benchmark_data_index_markdown(&index);
    bijux_dna_infra::atomic_create_bytes(&request.report_path, report.as_bytes())
        .with_context(|| format!("publish data-index report {}", request.report_path.display()))?;
    Ok(index)
}

#[must_use]
pub fn render_benchmark_data_index_markdown(index: &BenchmarkDataIndex) -> String {
    let eligibility = index
        .summary
        .benchmark_eligibility
        .iter()
        .map(|(status, count)| {
            let label = match status {
                BenchmarkEligibility::Eligible => "eligible",
                BenchmarkEligibility::Incomplete => "incomplete",
            };
            format!("**{label} {count}**")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let mut lines = vec![
        "# Governed scientific data index".to_string(),
        String::new(),
        "This report proves file identity and exposes provenance gaps. It does not".to_string(),
        "turn incomplete metadata into scientific acceptance.".to_string(),
        String::new(),
    ];
    if let (Some(path), Some(digest)) =
        (index.catalog_registry_path.as_deref(), index.catalog_registry_sha256.as_deref())
    {
        lines.push(format!("- Catalog registry: `{path}`"));
        lines.push(format!("- Catalog registry SHA-256: `{digest}`"));
    }
    lines.extend([
        format!("- Catalog records: **{}**", index.summary.record_count),
        format!("- Verified governed files: **{}**", index.summary.file_count),
        format!("- Verified bytes: **{}**", index.summary.size_bytes),
        format!("- Benchmark eligibility: {eligibility}"),
        String::new(),
        "| Data ID | Files | Integrity | Benchmark use | Missing provenance |".to_string(),
        "|---|---:|---|---|---|".to_string(),
    ]);
    for record in &index.records {
        let integrity = match record.integrity_status {
            DataIntegrityStatus::Verified => "verified",
            DataIntegrityStatus::Unverified => "unverified",
        };
        let eligibility = match record.benchmark_eligibility {
            BenchmarkEligibility::Eligible => "eligible",
            BenchmarkEligibility::Incomplete => "incomplete",
        };
        let missing = if record.missing_provenance.is_empty() {
            "-".to_string()
        } else {
            record.missing_provenance.join(", ")
        };
        lines.push(format!(
            "| `{}` | {} | {integrity} | {eligibility} | {missing} |",
            record.id,
            record.files.len()
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn normalize_record(
    catalog_path: &Path,
    data_root: &Path,
    files_by_name: &BTreeMap<String, Vec<PathBuf>>,
    registry_id: &str,
    snapshot: &GovernedDataCatalogSnapshot,
) -> Result<BenchmarkDataIndexRecord> {
    let metadata = fs::symlink_metadata(catalog_path)
        .with_context(|| format!("inspect data catalog {}", catalog_path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("data catalog is not a regular non-symlink file: {}", catalog_path.display());
    }
    let bytes = fs::read(catalog_path)
        .with_context(|| format!("read data catalog {}", catalog_path.display()))?;
    let value: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse data catalog {}", catalog_path.display()))?;
    let record = value.as_object().context("data catalog root must be an object")?;
    let id = required_text(record, "id", "data catalog")?;
    let schema_version = record.get("schema_version").and_then(Value::as_str);
    let catalog_sha256 = bijux_dna_infra::hash_bytes_sha256(&bytes);
    if !matches!(
        schema_version,
        Some(GOVERNED_DATA_CATALOG_SCHEMA_V1 | GOVERNED_DATA_CATALOG_SCHEMA_V2)
    ) || !valid_identifier(id)
        || catalog_path.file_stem().and_then(|value| value.to_str()) != Some(id)
        || id != registry_id
    {
        bail!("catalog snapshot identity is invalid: {}", catalog_path.display());
    }
    if schema_version != Some(snapshot.catalog_schema_version.as_str()) {
        bail!("catalog snapshot schema drifted: {}", catalog_path.display());
    }
    if catalog_sha256 != snapshot.catalog_sha256 {
        bail!("catalog snapshot digest drifted: {}", catalog_path.display());
    }
    if stable_data_path(catalog_path, data_root)? != snapshot.catalog_path {
        bail!("catalog snapshot path drifted: {}", catalog_path.display());
    }
    let purpose = required_text(record, "purpose", id)?.to_string();
    let access_classification = required_text(record, "access_classification", id)?.to_string();
    let redistribution_classification =
        required_text(record, "redistribution_classification", id)?.to_string();
    let known_limitations = required_string_list(record, "known_limitations", id)?;
    if known_limitations.is_empty() {
        bail!("{id} lacks explicit known limitations");
    }
    let files = governed_files(&value, data_root, files_by_name)?;
    let temporal_provenance = temporal_provenance(record)?;
    let source_identities = source_identities(record)?;
    let mut missing_provenance = Vec::new();
    if files.is_empty() {
        missing_provenance.push("governed_files".to_string());
    }
    if source_identities.is_empty() {
        missing_provenance.push("source_identity".to_string());
    }
    if temporal_provenance.timestamp.is_none() {
        missing_provenance.push("temporal_provenance".to_string());
    }
    let integrity_status = if files.is_empty() {
        DataIntegrityStatus::Unverified
    } else {
        DataIntegrityStatus::Verified
    };
    let benchmark_eligibility = if missing_provenance.is_empty() {
        BenchmarkEligibility::Eligible
    } else {
        BenchmarkEligibility::Incomplete
    };
    Ok(BenchmarkDataIndexRecord {
        id: id.to_string(),
        catalog_path: stable_data_path(catalog_path, data_root)?,
        catalog_sha256,
        purpose,
        source_identities,
        access_classification,
        redistribution_classification,
        scientific_name: optional_text(record, "scientific_name", id)?,
        reference_assembly: optional_text(record, "reference_assembly", id)?,
        temporal_provenance,
        files,
        known_limitations,
        integrity_status,
        benchmark_eligibility,
        missing_provenance,
    })
}

fn governed_files(
    value: &Value,
    data_root: &Path,
    files_by_name: &BTreeMap<String, Vec<PathBuf>>,
) -> Result<Vec<BenchmarkDataIndexFile>> {
    let mut found = BTreeMap::new();
    visit_file_contracts(value, "catalog", data_root, files_by_name, &mut found)?;
    Ok(found.into_values().collect())
}

fn visit_file_contracts(
    value: &Value,
    label: &str,
    data_root: &Path,
    files_by_name: &BTreeMap<String, Vec<PathBuf>>,
    found: &mut BTreeMap<String, BenchmarkDataIndexFile>,
) -> Result<()> {
    match value {
        Value::Object(object) => {
            if let Some(candidate) = file_contract(object, label, data_root, files_by_name)? {
                if let Some(previous) = found.get(&candidate.path) {
                    if previous.sha256 != candidate.sha256
                        || previous.size_bytes != candidate.size_bytes
                    {
                        bail!("conflicting data identities for {}", candidate.path);
                    }
                }
                found.insert(candidate.path.clone(), candidate);
            }
            for (key, child) in object {
                if !key.starts_with("source_") {
                    visit_file_contracts(
                        child,
                        &format!("{label}.{key}"),
                        data_root,
                        files_by_name,
                        found,
                    )?;
                }
            }
        }
        Value::Array(items) => {
            for (position, child) in items.iter().enumerate() {
                visit_file_contracts(
                    child,
                    &format!("{label}.{position}"),
                    data_root,
                    files_by_name,
                    found,
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn file_contract(
    value: &Map<String, Value>,
    label: &str,
    data_root: &Path,
    files_by_name: &BTreeMap<String, Vec<PathBuf>>,
) -> Result<Option<BenchmarkDataIndexFile>> {
    let has_path_key = PATH_CONTRACTS.iter().any(|(path, _, _)| value.contains_key(*path));
    let (path, digest, size, legacy_fields) =
        if ["name", "sha256", "size_bytes"].iter().all(|key| value.contains_key(*key))
            && !has_path_key
        {
            let name = required_text(value, "name", label)?;
            let matches = files_by_name.get(name).map(Vec::as_slice).unwrap_or_default();
            if matches.len() != 1 {
                bail!("bare data filename is not uniquely resolvable at {label}: {name}");
            }
            (
                matches[0].clone(),
                required_text(value, "sha256", label)?.to_string(),
                Some(required_u64(value, "size_bytes", label)?),
                vec!["name".to_string(), "sha256".to_string(), "size_bytes".to_string()],
            )
        } else if let Some((path_key, digest_key, size_key)) =
            PATH_CONTRACTS.iter().find(|(path, _, _)| value.contains_key(*path))
        {
            if !value.contains_key(*digest_key) {
                bail!("incomplete file identity at {label}");
            }
            let path = PathBuf::from(required_text(value, path_key, label)?);
            let digest = required_text(value, digest_key, label)?.to_string();
            let size =
                value.get(*size_key).map(|_| required_u64(value, size_key, label)).transpose()?;
            let legacy_fields = [*path_key, *digest_key, *size_key]
                .into_iter()
                .filter(|key| value.contains_key(*key))
                .map(str::to_string)
                .collect();
            (path, digest, size, legacy_fields)
        } else {
            return Ok(None);
        };
    if !path.is_absolute() {
        bail!("governed file path must be absolute at {label}: {}", path.display());
    }
    validate_digest(&digest, label)?;
    let path = governed_regular_file(&path, data_root, label)?;
    let observed_size = fs::metadata(&path)?.len();
    let size = size.unwrap_or(observed_size);
    if observed_size != size {
        bail!("governed data size drifted at {label}: {}", path.display());
    }
    let observed_digest = bijux_dna_infra::hash_file_sha256(&path)?;
    if observed_digest != digest {
        bail!("governed data digest drifted at {label}: {}", path.display());
    }
    let role = value
        .get("kind")
        .or_else(|| value.get("role"))
        .and_then(Value::as_str)
        .unwrap_or(label)
        .to_string();
    Ok(Some(BenchmarkDataIndexFile {
        path: stable_data_path(&path, data_root)?,
        sha256: digest,
        size_bytes: size,
        role,
        legacy_fields,
    }))
}

fn basename_index(data_root: &Path) -> Result<BTreeMap<String, Vec<PathBuf>>> {
    let mut index = BTreeMap::<String, Vec<PathBuf>>::new();
    let evidence_root = data_root.join("evidence");
    for entry in WalkDir::new(data_root)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| entry.path() != evidence_root)
    {
        let entry = entry.context("walk governed data root")?;
        if entry.file_type().is_symlink() {
            bail!("governed data root contains a symlink: {}", entry.path().display());
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry.path().strip_prefix(data_root)?;
        if relative.components().next().is_some_and(|component| component.as_os_str() == "catalog")
        {
            continue;
        }
        let name = entry
            .path()
            .file_name()
            .and_then(|value| value.to_str())
            .context("governed data filename is not UTF-8")?;
        index.entry(name.to_string()).or_default().push(entry.path().to_path_buf());
    }
    Ok(index)
}

fn temporal_provenance(record: &Map<String, Value>) -> Result<BenchmarkTemporalProvenance> {
    for field in TEMPORAL_FIELDS {
        if let Some(value) =
            record.get(field).and_then(Value::as_str).filter(|value| !value.is_empty())
        {
            validate_utc_timestamp(value, field)?;
            return Ok(BenchmarkTemporalProvenance {
                field: Some(field.to_string()),
                timestamp: Some(value.to_string()),
            });
        }
    }
    if let Some(acquisition) = record.get("acquisition").and_then(Value::as_object) {
        for field in TEMPORAL_FIELDS {
            if let Some(value) =
                acquisition.get(field).and_then(Value::as_str).filter(|value| !value.is_empty())
            {
                validate_utc_timestamp(value, field)?;
                return Ok(BenchmarkTemporalProvenance {
                    field: Some(format!("acquisition.{field}")),
                    timestamp: Some(value.to_string()),
                });
            }
        }
    }
    Ok(BenchmarkTemporalProvenance { field: None, timestamp: None })
}

fn source_identities(record: &Map<String, Value>) -> Result<Vec<String>> {
    let mut identities = Vec::new();
    for field in [
        "source",
        "source_url",
        "source_record",
        "source_page",
        "source_parent_id",
        "reference_parent_id",
    ] {
        if let Some(value) =
            record.get(field).and_then(Value::as_str).filter(|value| !value.is_empty())
        {
            identities.push(format!("{field}:{value}"));
        }
    }
    if let Some(parents) = record.get("parents") {
        let parents = parents.as_array().context("data catalog parents must be an array")?;
        for parent in parents {
            let identity = if let Some(parent) = parent.as_str() {
                parent
            } else {
                let parent = parent
                    .as_object()
                    .context("data catalog parent identity must be text or an object")?;
                ["path", "governed_path", "derived_path", "name"]
                    .into_iter()
                    .find_map(|field| parent.get(field).and_then(Value::as_str))
                    .context("structured data catalog parent lacks a text path or name")?
            };
            if !identity.is_empty() {
                identities.push(format!("parent:{identity}"));
            }
        }
    }
    Ok(identities)
}

fn summarize_records(records: &[BenchmarkDataIndexRecord]) -> Result<BenchmarkDataIndexSummary> {
    let mut file_count = 0_u64;
    let mut size_bytes = 0_u64;
    let mut integrity_verified_count = 0_u64;
    let mut benchmark_eligibility = BTreeMap::new();
    for record in records {
        file_count = file_count
            .checked_add(u64::try_from(record.files.len()).context("file count overflow")?)
            .context("file count overflow")?;
        size_bytes = record.files.iter().try_fold(size_bytes, |total, file| {
            total.checked_add(file.size_bytes).context("file size total overflow")
        })?;
        if record.integrity_status == DataIntegrityStatus::Verified {
            integrity_verified_count =
                integrity_verified_count.checked_add(1).context("verified count overflow")?;
        }
        let count = benchmark_eligibility.entry(record.benchmark_eligibility).or_insert(0_u64);
        *count = count.checked_add(1).context("eligibility count overflow")?;
    }
    Ok(BenchmarkDataIndexSummary {
        record_count: u64::try_from(records.len()).context("record count overflow")?,
        file_count,
        size_bytes,
        integrity_verified_count,
        benchmark_eligibility,
    })
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not a non-symlink directory: {}", path.display());
    }
    fs::canonicalize(path).with_context(|| format!("resolve {label} {}", path.display()))
}

fn governed_regular_file(path: &Path, data_root: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect governed file at {label}: {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!(
            "governed data file is not a regular non-symlink file at {label}: {}",
            path.display()
        );
    }
    let path = fs::canonicalize(path)?;
    require_under(&path, data_root, label)?;
    Ok(path)
}

fn stable_data_path(path: &Path, data_root: &Path) -> Result<String> {
    let path = fs::canonicalize(path)?;
    let relative = path.strip_prefix(data_root).with_context(|| {
        format!("governed data path escapes {}: {}", data_root.display(), path.display())
    })?;
    Ok(format!("study://data/{}", relative.to_string_lossy()))
}

fn resolve_study_data_uri(value: &str, data_root: &Path) -> Result<PathBuf> {
    let relative = value
        .strip_prefix("study://data/")
        .with_context(|| format!("catalog snapshot path is not study-data relative: {value}"))?;
    if relative.is_empty()
        || relative.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        bail!("catalog snapshot path is not normalized: {value}");
    }
    governed_regular_file(&data_root.join(relative), data_root, "catalog snapshot")
}

fn require_under(path: &Path, root: &Path, label: &str) -> Result<()> {
    if !path.starts_with(root) {
        bail!("{label} escapes governed root {}: {}", root.display(), path.display());
    }
    Ok(())
}

fn required_text<'a>(object: &'a Map<String, Value>, key: &str, label: &str) -> Result<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .with_context(|| format!("{label} lacks required {key}"))
}

fn optional_text(object: &Map<String, Value>, key: &str, label: &str) -> Result<Option<String>> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if !value.is_empty() => Ok(Some(value.clone())),
        _ => bail!("{label} contains invalid optional {key}"),
    }
}

fn required_string_list(
    object: &Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<Vec<String>> {
    object
        .get(key)
        .and_then(Value::as_array)
        .with_context(|| format!("{label} lacks required {key}"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .with_context(|| format!("{label} contains invalid {key}"))
        })
        .collect()
}

fn required_u64(object: &Map<String, Value>, key: &str, label: &str) -> Result<u64> {
    object
        .get(key)
        .and_then(Value::as_u64)
        .with_context(|| format!("{label}.{key} must be a non-negative integer"))
}

fn validate_digest(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("governed file digest is invalid at {label}");
    }
    Ok(())
}

fn validate_utc_timestamp(value: &str, label: &str) -> Result<()> {
    let timestamp = DateTime::<FixedOffset>::parse_from_rfc3339(value)
        .with_context(|| format!("{label} must be an RFC 3339 timestamp"))?;
    if timestamp.offset().local_minus_utc() != 0 {
        bail!("{label} must use UTC");
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}
