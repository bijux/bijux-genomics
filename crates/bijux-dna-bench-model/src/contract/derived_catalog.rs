//! Owner: bijux-dna-bench-model
//! Validation for checksum-bound data derived from governed scientific sources.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::diagnostics::BenchError;
use crate::model::{GovernedDerivedDataCatalog, GovernedDerivedDataCatalogSample};

use super::{
    materialization::validate_source_policy, validate_utc_timestamp,
    GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1,
};

/// Validate scientific context, lineage, sample strata, and every derived file identity.
///
/// # Errors
/// Returns an error when the catalog cannot authorize derived benchmark data consumption.
pub fn validate_governed_derived_data_catalog(
    catalog: &GovernedDerivedDataCatalog,
) -> Result<(), BenchError> {
    if catalog.schema_version != GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1 {
        return Err(invalid("governed derived data catalog schema mismatch"));
    }
    validate_identifier(&catalog.id, "catalog")?;
    for (value, label) in [
        (&catalog.purpose, "catalog purpose"),
        (&catalog.source, "catalog source"),
        (&catalog.scientific_name, "scientific name"),
        (&catalog.reference_assembly, "reference assembly"),
        (&catalog.derivation.pipeline_id, "derivation pipeline"),
        (&catalog.derivation.pipeline_version, "derivation pipeline version"),
        (&catalog.derivation.reference_id, "derivation reference"),
    ] {
        validate_nonempty(value, label)?;
    }
    validate_source_policy(&catalog.policy)?;
    validate_utc_timestamp(&catalog.admitted_at_utc, "derived catalog admission")?;
    validate_hex(&catalog.derivation.source_commit, 40, "derivation source commit")?;
    for (value, label) in [
        (&catalog.derivation.parameter_sha256, "derivation parameters"),
        (&catalog.derivation.reference_catalog_sha256, "derivation reference catalog"),
        (&catalog.derivation.contig_dictionary_sha256, "derivation contig dictionary"),
    ] {
        validate_hex(value, 64, label)?;
    }
    validate_ordered_nonempty_list(&catalog.allowed_uses, "allowed use")?;
    validate_sources(catalog)?;
    validate_samples_and_files(catalog)
}

fn validate_sources(catalog: &GovernedDerivedDataCatalog) -> Result<(), BenchError> {
    if catalog.sources.is_empty() {
        return Err(invalid("derived catalog contains no governed sources"));
    }
    let mut previous = None;
    for source in &catalog.sources {
        validate_identifier(&source.catalog_id, "source catalog")?;
        if previous.is_some_and(|value: &str| value >= source.catalog_id.as_str()) {
            return Err(invalid("derived catalog sources are not strictly ordered"));
        }
        previous = Some(source.catalog_id.as_str());
        let relative = source
            .catalog_path
            .strip_prefix("study://data/catalog/snapshots/")
            .ok_or_else(|| invalid("derived catalog source path is not source addressed"))?;
        let mut components = relative.split('/');
        let source_commit = components.next().unwrap_or_default();
        let file_name = components.next().unwrap_or_default();
        if components.next().is_some()
            || file_name != format!("{}.json", source.catalog_id)
            || validate_hex(source_commit, 40, "source catalog commit").is_err()
        {
            return Err(invalid("derived catalog source path is not source addressed"));
        }
        validate_hex(&source.catalog_sha256, 64, "source catalog")?;
    }
    Ok(())
}

fn validate_samples_and_files(catalog: &GovernedDerivedDataCatalog) -> Result<(), BenchError> {
    if catalog.samples.is_empty() || catalog.files.is_empty() {
        return Err(invalid("derived catalog must contain samples and files"));
    }
    let mut samples = BTreeMap::<&str, &GovernedDerivedDataCatalogSample>::new();
    let mut previous_sample = None;
    for sample in &catalog.samples {
        validate_identifier(&sample.sample_id, "derived sample")?;
        if previous_sample.is_some_and(|value: &str| value >= sample.sample_id.as_str()) {
            return Err(invalid("derived catalog samples are not strictly ordered"));
        }
        previous_sample = Some(sample.sample_id.as_str());
        validate_ordered_nonempty_list(&sample.source_accessions, "sample source accession")?;
        validate_nonempty(&sample.library_context, "sample library context")?;
        validate_nonempty(&sample.assay_context, "sample assay context")?;
        validate_nonempty(&sample.held_out_split, "sample held-out split")?;
        validate_ordered_nonempty_list(&sample.control_roles, "sample control role")?;
        validate_ordered_nonempty_list(&sample.truth_roles, "sample truth role")?;
        validate_ordered_nonempty_list(&sample.read_group_ids, "sample read group")?;
        if sample.strata.is_empty()
            || sample.strata.iter().any(|(key, value)| {
                validate_nonempty(key, "sample stratum").is_err()
                    || validate_nonempty(value, "sample stratum value").is_err()
            })
        {
            return Err(invalid("derived catalog sample strata are incomplete"));
        }
        if samples.insert(&sample.sample_id, sample).is_some() {
            return Err(invalid("derived catalog repeats a sample"));
        }
    }

    let mut previous_file = None;
    let mut paths = BTreeSet::new();
    let mut samples_with_files = BTreeSet::new();
    for file in &catalog.files {
        let order = (file.sample_id.as_str(), file.kind.as_str(), file.path.as_str());
        if previous_file.is_some_and(|value| value >= order) {
            return Err(invalid("derived catalog files are not strictly ordered"));
        }
        previous_file = Some(order);
        if !samples.contains_key(file.sample_id.as_str()) {
            return Err(invalid("derived catalog file names an unknown sample"));
        }
        validate_nonempty(&file.kind, "derived file kind")?;
        validate_absolute_path(&file.path)?;
        validate_hex(&file.sha256, 64, "derived file")?;
        if file.size_bytes == 0 {
            return Err(invalid("derived file size is zero"));
        }
        validate_ordered_nonempty_list(&file.source_sha256s, "derived file source")?;
        for source in &file.source_sha256s {
            validate_hex(source, 64, "derived file source")?;
        }
        if !paths.insert(file.path.as_str()) {
            return Err(invalid("derived catalog repeats a file path"));
        }
        samples_with_files.insert(file.sample_id.as_str());
    }
    if samples_with_files != samples.keys().copied().collect() {
        return Err(invalid("derived catalog contains a sample without files"));
    }
    Ok(())
}

fn validate_absolute_path(value: &str) -> Result<(), BenchError> {
    let path = Path::new(value);
    if !path.is_absolute()
        || value.ends_with('/')
        || value.split('/').any(|part| matches!(part, "." | ".."))
    {
        return Err(invalid("derived file path is not absolute and normalized"));
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
        return Err(invalid(&format!("{label} identity is malformed")));
    }
    Ok(())
}

fn validate_hex(value: &str, length: usize, label: &str) -> Result<(), BenchError> {
    if value.len() != length
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || value.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid(&format!("{label} identity is malformed")));
    }
    Ok(())
}

fn validate_nonempty(value: &str, label: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() || value != value.trim() {
        return Err(invalid(&format!("{label} is empty or malformed")));
    }
    Ok(())
}

fn validate_nonempty_list(values: &[String], label: &str) -> Result<(), BenchError> {
    if values.is_empty() {
        return Err(invalid(&format!("{label} list is empty")));
    }
    for value in values {
        validate_nonempty(value, label)?;
    }
    Ok(())
}

fn validate_ordered_nonempty_list(values: &[String], label: &str) -> Result<(), BenchError> {
    validate_nonempty_list(values, label)?;
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(invalid(&format!("{label} list is not strictly ordered")));
    }
    Ok(())
}

fn invalid(message: &str) -> BenchError {
    BenchError::InvalidPolicy(message.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::model::{
        BenchmarkCorpusClass, CorpusSourcePolicy, GovernedDataDerivation,
        GovernedDerivedDataCatalog, GovernedDerivedDataCatalogFile,
        GovernedDerivedDataCatalogSample, GovernedDerivedDataSource,
    };

    use super::validate_governed_derived_data_catalog;

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn catalog() -> GovernedDerivedDataCatalog {
        GovernedDerivedDataCatalog {
            schema_version: super::GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1.to_string(),
            id: "ancient-dna-bam-diversity".to_string(),
            purpose: "Admit a real ancient-DNA BAM diversity cohort".to_string(),
            source: "governed FASTQ and reference catalogs".to_string(),
            policy: CorpusSourcePolicy {
                class: BenchmarkCorpusClass::ProductionReal,
                access_classification: "public".to_string(),
                redistribution_classification: "derived-results-only".to_string(),
                access_terms_url: "https://example.org/terms".to_string(),
                attribution: "Cite the source studies".to_string(),
                projected_storage_bytes: 1,
                projected_cpu_hours: 1,
                projection_basis: "One bounded cohort".to_string(),
                truth_roles: vec!["bam_diversity".to_string()],
                truth_strategy: "orthogonal_authentication".to_string(),
                known_limitations: vec!["Alignment parameters affect observations".to_string()],
            },
            sources: vec![GovernedDerivedDataSource {
                catalog_id: "human-short-read-diversity".to_string(),
                catalog_path: format!(
                    "study://data/catalog/snapshots/{}/human-short-read-diversity.json",
                    "7".repeat(40)
                ),
                catalog_sha256: digest('1'),
            }],
            admitted_at_utc: "2026-08-14T16:00:00Z".to_string(),
            scientific_name: "Homo sapiens".to_string(),
            reference_assembly: "GRCh38".to_string(),
            allowed_uses: vec!["comparative_benchmarking".to_string()],
            derivation: GovernedDataDerivation {
                pipeline_id: "ancient-dna-bam-materialization".to_string(),
                pipeline_version: "1.0.0".to_string(),
                source_commit: "a".repeat(40),
                parameter_sha256: digest('2'),
                reference_id: "grch38-reference".to_string(),
                reference_catalog_sha256: digest('3'),
                contig_dictionary_sha256: digest('4'),
            },
            samples: vec![GovernedDerivedDataCatalogSample {
                sample_id: "err769610".to_string(),
                source_accessions: vec!["ERR769610".to_string()],
                library_context: "ancient single-end shotgun library".to_string(),
                assay_context: "whole-genome sequencing".to_string(),
                strata: BTreeMap::from([("era".to_string(), "ancient".to_string())]),
                control_roles: vec!["none".to_string()],
                truth_roles: vec!["orthogonal_ancient_authentication".to_string()],
                held_out_split: "evaluation".to_string(),
                read_group_ids: vec!["ERR769610".to_string()],
            }],
            files: vec![GovernedDerivedDataCatalogFile {
                path: "/study/data/derived/ERR769610.bam".to_string(),
                sha256: digest('5'),
                size_bytes: 1,
                kind: "coordinate-sorted-deduplicated-bam".to_string(),
                sample_id: "err769610".to_string(),
                source_sha256s: vec![digest('6')],
            }],
        }
    }

    #[test]
    fn accepts_complete_derived_scientific_lineage() {
        validate_governed_derived_data_catalog(&catalog()).expect("valid derived catalog");
    }

    #[test]
    fn rejects_derived_file_without_source_identity() {
        let mut catalog = catalog();
        catalog.files[0].source_sha256s.clear();
        assert!(validate_governed_derived_data_catalog(&catalog).is_err());
    }

    #[test]
    fn rejects_source_catalog_without_commit_addressing() {
        let mut catalog = catalog();
        catalog.sources[0].catalog_path =
            "study://data/catalog/snapshots/source/human-short-read-diversity.json".to_string();
        assert!(validate_governed_derived_data_catalog(&catalog).is_err());
    }

    #[test]
    fn rejects_sample_without_explicit_control_role() {
        let mut catalog = catalog();
        catalog.samples[0].control_roles.clear();
        assert!(validate_governed_derived_data_catalog(&catalog).is_err());
    }
}
