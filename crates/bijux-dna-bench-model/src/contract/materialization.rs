//! Owner: bijux-dna-bench-model
//! Fail-closed validation for governed corpus acquisition and admission records.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, FixedOffset};

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkCorpusClass, CorpusAcquisitionFile, CorpusAcquisitionLayout, CorpusAcquisitionPlan,
    CorpusSourcePolicy, FastqCorpusSourceSpec, GovernedDataCatalog, GovernedDataCatalogDocument,
    GovernedDataCatalogFile, GovernedPreservedDataCatalog,
};

use super::{
    CORPUS_ACQUISITION_SCHEMA_V2, FASTQ_CORPUS_SOURCE_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V2,
};

/// Validate a production FASTQ source declaration before archive resolution or acquisition.
///
/// # Errors
/// Returns an error when identity, production class, access, truth, projection, or sample closure
/// is incomplete or internally inconsistent.
pub fn validate_fastq_corpus_source_spec(spec: &FastqCorpusSourceSpec) -> Result<(), BenchError> {
    if spec.schema_version != FASTQ_CORPUS_SOURCE_SCHEMA_V1 {
        return Err(invalid("FASTQ corpus source schema mismatch"));
    }
    validate_identifier(&spec.corpus_id, "corpus")?;
    validate_identifier(&spec.portfolio_id, "portfolio")?;
    if !matches!(
        spec.policy.class,
        BenchmarkCorpusClass::ProductionReal | BenchmarkCorpusClass::QualificationReal
    ) {
        return Err(invalid("FASTQ corpus source class is not publishable real data"));
    }
    for (value, label) in [
        (&spec.species, "species"),
        (&spec.description, "description"),
        (&spec.source_authority, "source authority"),
        (&spec.policy.access_classification, "access classification"),
        (&spec.policy.redistribution_classification, "redistribution classification"),
        (&spec.policy.attribution, "attribution"),
        (&spec.policy.projection_basis, "projection basis"),
        (&spec.policy.truth_strategy, "truth strategy"),
    ] {
        validate_nonempty(value, label)?;
    }
    for (value, label) in [
        (spec.species_id.as_deref(), "species identifier"),
        (spec.preferred_root.as_deref(), "preferred root"),
    ] {
        if let Some(value) = value {
            validate_nonempty(value, label)?;
        }
    }
    validate_https(&spec.policy.access_terms_url, "access terms")?;
    if spec.policy.projected_storage_bytes == 0 || spec.policy.projected_cpu_hours == 0 {
        return Err(invalid("FASTQ corpus storage and compute projections must be positive"));
    }
    validate_nonempty_list(&spec.policy.truth_roles, "truth role")?;
    validate_nonempty_list(&spec.policy.known_limitations, "known limitation")?;
    if spec.samples.is_empty() || spec.samples.len() != spec.target_total {
        return Err(invalid("FASTQ corpus target total disagrees with samples"));
    }
    let cohort_targets = [
        spec.target_ancient_se,
        spec.target_ancient_pe,
        spec.target_modern_se,
        spec.target_modern_pe,
    ];
    if cohort_targets.iter().any(Option::is_some)
        && (cohort_targets.iter().any(Option::is_none)
            || cohort_targets.into_iter().flatten().sum::<usize>() != spec.target_total)
    {
        return Err(invalid("FASTQ corpus cohort targets do not close over target total"));
    }
    let mut accessions = BTreeSet::new();
    for sample in &spec.samples {
        if !accessions.insert(sample.accession.as_str()) {
            return Err(invalid("FASTQ corpus source repeats a run accession"));
        }
        for (value, label) in [
            (&sample.accession, "run accession"),
            (&sample.study_accession, "study accession"),
            (&sample.era, "era"),
            (&sample.size_band, "size band"),
            (&sample.reason, "selection reason"),
        ] {
            validate_nonempty(value, label)?;
        }
        if !matches!(sample.era.as_str(), "ancient" | "modern") {
            return Err(invalid("FASTQ corpus sample era is unsupported"));
        }
    }
    Ok(())
}

pub(super) fn validate_source_policy(policy: &CorpusSourcePolicy) -> Result<(), BenchError> {
    if !matches!(
        policy.class,
        BenchmarkCorpusClass::ProductionReal | BenchmarkCorpusClass::QualificationReal
    ) {
        return Err(invalid("corpus source policy class is not publishable real data"));
    }
    for (value, label) in [
        (&policy.access_classification, "access classification"),
        (&policy.redistribution_classification, "redistribution classification"),
        (&policy.attribution, "attribution"),
        (&policy.projection_basis, "projection basis"),
        (&policy.truth_strategy, "truth strategy"),
    ] {
        validate_nonempty(value, label)?;
    }
    validate_https(&policy.access_terms_url, "access terms")?;
    if policy.projected_storage_bytes == 0 || policy.projected_cpu_hours == 0 {
        return Err(invalid("corpus source storage and compute projections must be positive"));
    }
    validate_nonempty_list(&policy.truth_roles, "truth role")?;
    validate_nonempty_list(&policy.known_limitations, "known limitation")?;
    Ok(())
}

/// Validate acquisition identity, source checksums, sample context, and aggregate closure.
///
/// # Errors
/// Returns an error when a source plan cannot safely authorize corpus materialization.
pub fn validate_corpus_acquisition_plan(plan: &CorpusAcquisitionPlan) -> Result<(), BenchError> {
    if plan.schema_version != CORPUS_ACQUISITION_SCHEMA_V2 {
        return Err(invalid("corpus acquisition schema mismatch"));
    }
    validate_identifier(&plan.corpus_id, "corpus")?;
    validate_identifier(&plan.portfolio_id, "portfolio")?;
    for (value, label) in [
        (&plan.species, "species"),
        (&plan.source_authority, "source authority"),
        (&plan.source_spec, "source specification"),
        (&plan.description, "description"),
    ] {
        validate_nonempty(value, label)?;
    }
    validate_utc_timestamp(&plan.retrieved_at_utc, "retrieval")?;
    validate_sha256(&plan.source_spec_sha256, "source specification")?;
    validate_source_policy(&plan.policy)?;
    if plan.samples.is_empty() {
        return Err(invalid("corpus acquisition contains no samples"));
    }

    let mut accessions = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut file_count = 0_u64;
    let mut total_size_bytes = 0_u64;
    for sample in &plan.samples {
        if !accessions.insert(sample.accession.as_str()) {
            return Err(invalid("corpus acquisition repeats a sample accession"));
        }
        for (value, label) in [
            (&sample.accession, "run accession"),
            (&sample.study_accession, "study accession"),
            (&sample.sample_accession, "sample accession"),
            (&sample.experiment_accession, "experiment accession"),
            (&sample.era, "sample era"),
            (&sample.size_band, "sample size band"),
            (&sample.selection_reason, "sample selection reason"),
            (&sample.scientific_name, "sample scientific name"),
            (&sample.library_source, "library source"),
            (&sample.library_strategy, "library strategy"),
            (&sample.library_selection, "library selection"),
            (&sample.instrument_platform, "instrument platform"),
            (&sample.instrument_model, "instrument model"),
            (&sample.first_public, "first publication"),
            (&sample.last_updated, "last update"),
        ] {
            validate_nonempty(value, label)?;
        }
        if sample.scientific_name != plan.species {
            return Err(invalid("sample species differs from acquisition species"));
        }
        if sample.archive_base_count == 0 || sample.archive_read_count == 0 {
            return Err(invalid("archive read and base counts must be positive"));
        }
        validate_https(&sample.source_report_url, "source report")?;
        validate_layout_files(sample.layout, &sample.files)?;
        for file in &sample.files {
            validate_acquisition_file(file)?;
            if !targets.insert((sample.accession.as_str(), file.name.as_str())) {
                return Err(invalid("corpus acquisition repeats a file target"));
            }
            file_count =
                file_count.checked_add(1).ok_or_else(|| invalid("file count overflows"))?;
            total_size_bytes = total_size_bytes
                .checked_add(file.size_bytes)
                .ok_or_else(|| invalid("file size total overflows"))?;
        }
    }
    let sample_count =
        u64::try_from(plan.samples.len()).map_err(|_| invalid("sample count overflows"))?;
    if plan.sample_count != sample_count
        || plan.file_count != file_count
        || plan.total_size_bytes != total_size_bytes
    {
        return Err(invalid("corpus acquisition aggregates do not close over samples"));
    }
    Ok(())
}

/// Validate an admitted catalog's provenance, scientific context, and file identities.
///
/// # Errors
/// Returns an error when the catalog cannot authorize benchmark corpus consumption.
pub fn validate_governed_data_catalog(catalog: &GovernedDataCatalog) -> Result<(), BenchError> {
    if catalog.schema_version != GOVERNED_DATA_CATALOG_SCHEMA_V2 {
        return Err(invalid("governed data catalog schema mismatch"));
    }
    validate_identifier(&catalog.id, "catalog")?;
    for (value, label) in [
        (&catalog.purpose, "catalog purpose"),
        (&catalog.source, "catalog source"),
        (&catalog.scientific_name, "scientific name"),
    ] {
        validate_nonempty(value, label)?;
    }
    validate_study_data_uri(&catalog.source_record, "source record")?;
    validate_sha256(&catalog.source_record_sha256, "source record")?;
    validate_utc_timestamp(&catalog.retrieved_at_utc, "retrieval")?;
    validate_utc_timestamp(&catalog.admitted_at_utc, "admission")?;
    validate_source_policy(&catalog.policy)?;
    if catalog.samples.is_empty() || catalog.files.is_empty() {
        return Err(invalid("governed data catalog must contain samples and files"));
    }

    let mut samples = BTreeMap::new();
    for sample in &catalog.samples {
        if samples.insert(sample.accession.as_str(), sample).is_some() {
            return Err(invalid("governed data catalog repeats a sample accession"));
        }
        for value in [
            &sample.accession,
            &sample.study_accession,
            &sample.sample_accession,
            &sample.experiment_accession,
            &sample.era,
            &sample.size_band,
            &sample.selection_reason,
            &sample.scientific_name,
            &sample.library_source,
            &sample.library_strategy,
            &sample.library_selection,
            &sample.instrument_platform,
            &sample.instrument_model,
            &sample.first_public,
            &sample.last_updated,
        ] {
            validate_nonempty(value, "catalog sample context")?;
        }
        if sample.scientific_name != catalog.scientific_name
            || sample.archive_base_count == 0
            || sample.archive_read_count == 0
        {
            return Err(invalid("governed data catalog sample context is inconsistent"));
        }
    }

    let mut paths = BTreeSet::new();
    let mut files_by_sample = BTreeMap::<&str, Vec<&GovernedDataCatalogFile>>::new();
    for file in &catalog.files {
        validate_catalog_file(file)?;
        let sample = samples
            .get(file.accession.as_str())
            .ok_or_else(|| invalid("catalog file refers to an unknown sample"))?;
        validate_catalog_file_sample(file, sample)?;
        if !paths.insert(file.path.as_str()) {
            return Err(invalid("governed data catalog repeats a file path"));
        }
        files_by_sample.entry(&file.accession).or_default().push(file);
    }
    for (accession, sample) in samples {
        let files = files_by_sample
            .get(accession)
            .ok_or_else(|| invalid("catalog sample has no FASTQ files"))?;
        let mates = files.iter().map(|file| file.mate).collect::<BTreeSet<_>>();
        let valid = match sample.layout {
            CorpusAcquisitionLayout::Se => files.len() == 1 && mates == BTreeSet::from([None]),
            CorpusAcquisitionLayout::Pe => {
                files.len() == 2 && mates == BTreeSet::from([Some(1), Some(2)])
            }
        };
        if !valid {
            return Err(invalid(
                "catalog file count or mate identity disagrees with sample layout",
            ));
        }
    }
    Ok(())
}

/// Validate either native v2 materialization or an exact preservation of a historical catalog.
///
/// # Errors
/// Returns an error when either representation violates its identity or provenance contract.
pub fn validate_governed_data_catalog_document(
    catalog: &GovernedDataCatalogDocument,
) -> Result<(), BenchError> {
    match catalog {
        GovernedDataCatalogDocument::Materialized(catalog) => {
            validate_governed_data_catalog(catalog)
        }
        GovernedDataCatalogDocument::Preserved(catalog) => {
            validate_governed_preserved_data_catalog(catalog)
        }
    }
}

/// Validate the preservation envelope used for heterogeneous v1 catalog authorities.
///
/// # Errors
/// Returns an error when the source identity, timestamp, or embedded v1 document is malformed.
pub fn validate_governed_preserved_data_catalog(
    catalog: &GovernedPreservedDataCatalog,
) -> Result<(), BenchError> {
    if catalog.schema_version != GOVERNED_DATA_CATALOG_SCHEMA_V2 {
        return Err(invalid("preserved data catalog schema mismatch"));
    }
    validate_catalog_identifier(&catalog.id)?;
    let expected_path = format!("study://data/catalog/{}.json", catalog.id);
    if catalog.source_catalog_path != expected_path {
        return Err(invalid("preserved data catalog source path disagrees with identity"));
    }
    validate_sha256(&catalog.source_catalog_sha256, "source catalog")?;
    if catalog.source_catalog_size_bytes == 0 {
        return Err(invalid("preserved data catalog source size is zero"));
    }
    if catalog.source_catalog_schema_version != super::GOVERNED_DATA_CATALOG_SCHEMA_V1 {
        return Err(invalid("preserved data catalog source schema is not v1"));
    }
    validate_utc_timestamp(&catalog.preserved_at_utc, "catalog preservation")?;
    let preserved = catalog
        .preserved_catalog
        .as_object()
        .ok_or_else(|| invalid("preserved data catalog payload is not an object"))?;
    if preserved.get("schema_version").and_then(serde_json::Value::as_str)
        != Some(super::GOVERNED_DATA_CATALOG_SCHEMA_V1)
        || preserved.get("id").and_then(serde_json::Value::as_str) != Some(catalog.id.as_str())
    {
        return Err(invalid("preserved data catalog payload identity disagrees with envelope"));
    }
    for field in ["purpose", "access_classification", "redistribution_classification"] {
        if preserved.get(field).and_then(serde_json::Value::as_str).is_none_or(str::is_empty) {
            return Err(invalid("preserved data catalog payload lacks required policy fields"));
        }
    }
    if preserved
        .get("known_limitations")
        .and_then(serde_json::Value::as_array)
        .is_none_or(Vec::is_empty)
    {
        return Err(invalid("preserved data catalog payload lacks known limitations"));
    }
    Ok(())
}

fn validate_layout_files(
    layout: CorpusAcquisitionLayout,
    files: &[CorpusAcquisitionFile],
) -> Result<(), BenchError> {
    match layout {
        CorpusAcquisitionLayout::Se if files.len() == 1 && files[0].mate.is_none() => Ok(()),
        CorpusAcquisitionLayout::Pe
            if files.len() == 2
                && files.iter().map(|file| file.mate).collect::<BTreeSet<_>>()
                    == BTreeSet::from([Some(1), Some(2)]) =>
        {
            Ok(())
        }
        _ => Err(invalid("acquisition file count or mate identity disagrees with layout")),
    }
}

fn validate_catalog_identifier(value: &str) -> Result<(), BenchError> {
    let mut bytes = value.bytes();
    if !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        || !bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        return Err(invalid("preserved data catalog identity is malformed"));
    }
    Ok(())
}

fn validate_acquisition_file(file: &CorpusAcquisitionFile) -> Result<(), BenchError> {
    validate_fastq_name(&file.name)?;
    validate_https(&file.source_url, "FASTQ source")?;
    validate_md5(&file.source_md5)?;
    if file.size_bytes == 0 {
        return Err(invalid("acquisition FASTQ size must be positive"));
    }
    Ok(())
}

fn validate_catalog_file(file: &GovernedDataCatalogFile) -> Result<(), BenchError> {
    let relative = file
        .path
        .strip_prefix('/')
        .ok_or_else(|| invalid("catalog FASTQ path is not absolute and normalized"))?;
    if relative.is_empty()
        || relative.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(invalid("catalog FASTQ path is not absolute and normalized"));
    }
    if !file.path.ends_with(".fastq.gz") || file.kind != "raw-fastq" {
        return Err(invalid("catalog file is not a raw compressed FASTQ"));
    }
    validate_sha256(&file.sha256, "catalog FASTQ")?;
    validate_md5(&file.source_md5)?;
    validate_https(&file.source_url, "FASTQ source")?;
    if file.size_bytes == 0 || file.read_count == 0 || file.base_count == 0 {
        return Err(invalid("catalog FASTQ size, reads, and bases must be positive"));
    }
    match (file.layout, file.mate) {
        (CorpusAcquisitionLayout::Se, None) | (CorpusAcquisitionLayout::Pe, Some(1 | 2)) => Ok(()),
        _ => Err(invalid("catalog FASTQ mate identity disagrees with layout")),
    }
}

fn validate_catalog_file_sample(
    file: &GovernedDataCatalogFile,
    sample: &crate::model::GovernedDataCatalogSample,
) -> Result<(), BenchError> {
    if file.study_accession != sample.study_accession
        || file.sample_accession != sample.sample_accession
        || file.experiment_accession != sample.experiment_accession
        || file.era != sample.era
        || file.layout != sample.layout
    {
        return Err(invalid("catalog file context differs from its sample"));
    }
    Ok(())
}

fn validate_fastq_name(value: &str) -> Result<(), BenchError> {
    if value.is_empty()
        || !value.ends_with(".fastq.gz")
        || value.contains('/')
        || value.contains('\\')
        || matches!(value, "." | "..")
    {
        return Err(invalid("acquisition FASTQ name is unsafe"));
    }
    Ok(())
}

pub(super) fn validate_identifier(value: &str, label: &str) -> Result<(), BenchError> {
    let mut bytes = value.bytes();
    if !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(invalid(&format!("{label} identity is malformed")));
    }
    Ok(())
}

pub(super) fn validate_nonempty(value: &str, label: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() || value != value.trim() {
        return Err(invalid(&format!("{label} is empty or malformed")));
    }
    Ok(())
}

pub(super) fn validate_nonempty_list(values: &[String], label: &str) -> Result<(), BenchError> {
    if values.is_empty() {
        return Err(invalid(&format!("{label} list is empty")));
    }
    for value in values {
        validate_nonempty(value, label)?;
    }
    Ok(())
}

pub(super) fn validate_https(value: &str, label: &str) -> Result<(), BenchError> {
    if value.strip_prefix("https://").is_none_or(str::is_empty)
        || value.chars().any(|ch| ch.is_ascii_whitespace() || ch == '\0')
    {
        return Err(invalid(&format!("{label} is not an HTTPS URL")));
    }
    Ok(())
}

fn validate_md5(value: &str) -> Result<(), BenchError> {
    if !is_lower_hex(value, 32) {
        return Err(invalid("acquisition MD5 identity is malformed"));
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<(), BenchError> {
    if !is_lower_hex(value, 64) || value.bytes().all(|byte| byte == b'0') {
        return Err(invalid(&format!("{label} SHA-256 identity is malformed")));
    }
    Ok(())
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Validate one RFC 3339 timestamp with a zero UTC offset.
///
/// # Errors
/// Returns an error when the value is malformed or carries a non-UTC offset.
pub fn validate_utc_timestamp(value: &str, label: &str) -> Result<(), BenchError> {
    let parsed = DateTime::<FixedOffset>::parse_from_rfc3339(value)
        .map_err(|_| invalid(&format!("{label} timestamp is not RFC 3339")))?;
    if parsed.offset().local_minus_utc() != 0 {
        return Err(invalid(&format!("{label} timestamp is not UTC")));
    }
    Ok(())
}

fn validate_study_data_uri(value: &str, label: &str) -> Result<(), BenchError> {
    let relative = value
        .strip_prefix("study://data/")
        .ok_or_else(|| invalid(&format!("{label} is not study-data relative")))?;
    if relative.is_empty()
        || relative.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(invalid(&format!("{label} is not normalized")));
    }
    Ok(())
}

fn invalid(message: &str) -> BenchError {
    BenchError::InvalidPolicy(message.to_string())
}
