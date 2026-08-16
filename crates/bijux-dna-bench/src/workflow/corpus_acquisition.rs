//! Owner: bijux-dna-bench
//! Typed ENA metadata resolution and immutable corpus acquisition-plan publication.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_corpus_acquisition_plan, validate_fastq_corpus_source_spec,
    BenchmarkDataSourceCollection, BenchmarkDataSourceSpec, CorpusAcquisitionFile,
    CorpusAcquisitionLayout, CorpusAcquisitionPlan, CorpusAcquisitionSample,
    FastqCorpusSourceSample, FastqCorpusSourceSpec, CORPUS_ACQUISITION_SCHEMA_V2,
};
use bijux_dna_db_ena::{
    EnaClient, EnaFileSource, EnaQuery, EnaRecord, EnaResultKind, EnaSourcePreference,
};
use chrono::{DateTime, FixedOffset};
const MAX_CORPUS_SPEC_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusAcquisitionRequest {
    pub spec_path: PathBuf,
    pub output_path: PathBuf,
    pub retrieved_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedFastqCollectionAcquisitionRequest {
    pub spec_path: PathBuf,
    pub collection_id: String,
    pub output_path: PathBuf,
    pub retrieved_at_utc: String,
}

/// Resolve live ENA metadata and immutably publish one validated acquisition plan.
///
/// # Errors
/// Returns an error for malformed specifications, network or ENA ambiguity, incomplete metadata,
/// unsafe file identities, invalid timestamps, or an existing output authority.
pub fn acquire_fastq_corpus(request: &CorpusAcquisitionRequest) -> Result<CorpusAcquisitionPlan> {
    match std::fs::symlink_metadata(&request.output_path) {
        Ok(_) => {
            bail!("refusing to replace acquisition plan: {}", request.output_path.display());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| {
                format!("inspect acquisition plan output {}", request.output_path.display())
            });
        }
    }
    let spec = load_fastq_corpus_source_spec(&request.spec_path)?;
    let query = EnaQuery {
        projects: Vec::new(),
        samples: Vec::new(),
        extra_accessions: spec.samples.iter().map(|sample| sample.accession.clone()).collect(),
        result: EnaResultKind::ReadRun,
    };
    let records = EnaClient::from_crate_identity()
        .context("create ENA acquisition client")?
        .fetch_records(&query)
        .context("fetch ENA corpus metadata")?;
    let plan = derive_plan(&request.spec_path, &spec, &records, &request.retrieved_at_utc)?;
    bijux_dna_infra::atomic_create_json(&request.output_path, &plan)
        .with_context(|| format!("publish acquisition plan {}", request.output_path.display()))?;
    Ok(plan)
}

/// Resolve one explicitly selected FASTQ collection from a domain-neutral source authority.
///
/// Each collection becomes its own acquisition plan so a portfolio can retain multiple species,
/// archive studies, assay contexts, and layouts without weakening the single-species governed
/// catalog contract.
///
/// # Errors
/// Returns an error for an unsafe output, incomplete selection, missing collection, archive
/// ambiguity, mixed species inside one collection, or invalid checksum-bearing metadata.
pub fn acquire_selected_fastq_collection(
    request: &SelectedFastqCollectionAcquisitionRequest,
) -> Result<CorpusAcquisitionPlan> {
    refuse_existing_output(&request.output_path)?;
    let spec = super::load_benchmark_data_source_spec(&request.spec_path)?;
    let collection = selected_fastq_collection(&spec, &request.collection_id)?;
    let query = EnaQuery {
        projects: Vec::new(),
        samples: Vec::new(),
        extra_accessions: collection.selected_accessions.clone(),
        result: EnaResultKind::ReadRun,
    };
    let records = EnaClient::from_crate_identity()
        .context("create ENA acquisition client")?
        .fetch_records(&query)
        .context("fetch selected collection metadata")?;
    let plan = derive_selected_fastq_collection_acquisition_plan(
        &request.spec_path,
        &request.collection_id,
        &records,
        &request.retrieved_at_utc,
    )?;
    bijux_dna_infra::atomic_create_json(&request.output_path, &plan)
        .with_context(|| format!("publish acquisition plan {}", request.output_path.display()))?;
    Ok(plan)
}

/// Deterministically derive a selected-collection plan from caller-supplied ENA records.
///
/// # Errors
/// Returns an error under the same identity, selection, species, and metadata conditions as live
/// acquisition.
pub fn derive_selected_fastq_collection_acquisition_plan(
    spec_path: &Path,
    collection_id: &str,
    records: &[EnaRecord],
    retrieved_at_utc: &str,
) -> Result<CorpusAcquisitionPlan> {
    let spec = super::load_benchmark_data_source_spec(spec_path)?;
    let collection = selected_fastq_collection(&spec, collection_id)?;
    derive_selected_collection_plan(spec_path, &spec, collection, records, retrieved_at_utc)
}

/// Derive a validated plan from caller-supplied ENA records for deterministic offline replay.
///
/// # Errors
/// Returns an error under the same identity and metadata conditions as live acquisition.
pub fn derive_fastq_corpus_acquisition_plan(
    spec_path: &Path,
    records: &[EnaRecord],
    retrieved_at_utc: &str,
) -> Result<CorpusAcquisitionPlan> {
    let spec = load_fastq_corpus_source_spec(spec_path)?;
    derive_plan(spec_path, &spec, records, retrieved_at_utc)
}

/// Load and validate a governed production FASTQ source declaration.
///
/// # Errors
/// Returns an error when the declaration is unreadable, malformed, or violates source policy.
pub fn load_fastq_corpus_source_spec(path: &Path) -> Result<FastqCorpusSourceSpec> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_CORPUS_SPEC_BYTES)
        .with_context(|| format!("read corpus specification {}", path.display()))?;
    let spec: FastqCorpusSourceSpec =
        toml::from_str(std::str::from_utf8(&bytes).context("corpus specification is not UTF-8")?)
            .with_context(|| format!("parse corpus specification {}", path.display()))?;
    validate_fastq_corpus_source_spec(&spec).context("validate FASTQ corpus source")?;
    Ok(spec)
}

fn derive_plan(
    spec_path: &Path,
    spec: &FastqCorpusSourceSpec,
    records: &[EnaRecord],
    retrieved_at_utc: &str,
) -> Result<CorpusAcquisitionPlan> {
    if spec.source_authority != "European Nucleotide Archive" {
        bail!("FASTQ acquisition source authority is not European Nucleotide Archive");
    }
    let retrieved_at = DateTime::<FixedOffset>::parse_from_rfc3339(retrieved_at_utc)
        .context("corpus retrieval timestamp is not RFC 3339")?;
    if retrieved_at.offset().local_minus_utc() != 0 {
        bail!("corpus retrieval timestamp must use UTC");
    }
    let mut by_accession = BTreeMap::new();
    for record in records {
        let accession = required(&record.run_accession, "run accession")?;
        if by_accession.insert(accession, record).is_some() {
            bail!("ENA returned duplicate run metadata for {accession}");
        }
    }
    let expected =
        spec.samples.iter().map(|sample| sample.accession.as_str()).collect::<BTreeSet<_>>();
    if by_accession.keys().copied().collect::<BTreeSet<_>>() != expected {
        bail!("ENA run metadata does not exactly match the corpus specification");
    }

    let samples = spec
        .samples
        .iter()
        .map(|sample| {
            let record = by_accession
                .get(sample.accession.as_str())
                .context("resolved ENA record is absent after closure validation")?;
            resolve_sample(spec, sample, record)
        })
        .collect::<Result<Vec<_>>>()?;
    let file_count = samples.iter().try_fold(0_u64, |count, sample| {
        count
            .checked_add(u64::try_from(sample.files.len())?)
            .context("corpus acquisition file count overflows")
    })?;
    let total_size_bytes =
        samples.iter().flat_map(|sample| &sample.files).try_fold(0_u64, |total, file| {
            total.checked_add(file.size_bytes).context("corpus acquisition size overflows")
        })?;
    let plan = CorpusAcquisitionPlan {
        schema_version: CORPUS_ACQUISITION_SCHEMA_V2.to_string(),
        corpus_id: spec.corpus_id.clone(),
        portfolio_id: spec.portfolio_id.clone(),
        species: spec.species.clone(),
        retrieved_at_utc: retrieved_at_utc.to_string(),
        source_authority: spec.source_authority.clone(),
        description: spec.description.clone(),
        policy: spec.policy.clone(),
        evaluation: spec.evaluation.clone(),
        source_spec: spec_path.display().to_string(),
        source_spec_sha256: bijux_dna_infra::hash_file_sha256(spec_path)
            .context("hash corpus specification")?,
        sample_count: u64::try_from(samples.len())?,
        file_count,
        total_size_bytes,
        samples,
    };
    validate_corpus_acquisition_plan(&plan).context("validate derived acquisition plan")?;
    Ok(plan)
}

fn derive_selected_collection_plan(
    spec_path: &Path,
    spec: &BenchmarkDataSourceSpec,
    collection: &BenchmarkDataSourceCollection,
    records: &[EnaRecord],
    retrieved_at_utc: &str,
) -> Result<CorpusAcquisitionPlan> {
    require_utc(retrieved_at_utc)?;
    let by_accession = records_by_accession(records)?;
    let expected =
        collection.selected_accessions.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if by_accession.keys().copied().collect::<BTreeSet<_>>() != expected {
        bail!("ENA run metadata does not exactly match the selected source collection");
    }
    let species = expected
        .iter()
        .map(|accession| {
            let record = by_accession
                .get(accession)
                .context("selected ENA record is absent after closure validation")?;
            required(&record.scientific_name, "scientific name")
        })
        .collect::<Result<BTreeSet<_>>>()?;
    if species.len() != 1 {
        bail!("selected FASTQ collection contains more than one ENA scientific name");
    }
    let species = species
        .first()
        .context("selected FASTQ collection contains no scientific name")?
        .to_string();
    let samples = collection
        .selected_accessions
        .iter()
        .map(|accession| {
            let record = by_accession
                .get(accession.as_str())
                .context("selected ENA record is absent after closure validation")?;
            resolve_selected_collection_sample(collection, accession, record, &species)
        })
        .collect::<Result<Vec<_>>>()?;
    let file_count = samples.iter().try_fold(0_u64, |count, sample| {
        count
            .checked_add(u64::try_from(sample.files.len())?)
            .context("corpus acquisition file count overflows")
    })?;
    let total_size_bytes =
        samples.iter().flat_map(|sample| &sample.files).try_fold(0_u64, |total, file| {
            total.checked_add(file.size_bytes).context("corpus acquisition size overflows")
        })?;
    let plan = CorpusAcquisitionPlan {
        schema_version: CORPUS_ACQUISITION_SCHEMA_V2.to_string(),
        corpus_id: format!("{}-{}", spec.portfolio_id, collection.id),
        portfolio_id: spec.portfolio_id.clone(),
        species,
        retrieved_at_utc: retrieved_at_utc.to_string(),
        source_authority: "European Nucleotide Archive".to_string(),
        description: format!("{} Selection: {}", spec.description, collection.selection_scope),
        policy: spec.policy.clone(),
        evaluation: spec.evaluation.clone(),
        source_spec: spec_path.display().to_string(),
        source_spec_sha256: bijux_dna_infra::hash_file_sha256(spec_path)
            .context("hash benchmark data source specification")?,
        sample_count: u64::try_from(samples.len())?,
        file_count,
        total_size_bytes,
        samples,
    };
    validate_corpus_acquisition_plan(&plan).context("validate selected collection plan")?;
    Ok(plan)
}

fn selected_fastq_collection<'a>(
    spec: &'a BenchmarkDataSourceSpec,
    collection_id: &str,
) -> Result<&'a BenchmarkDataSourceCollection> {
    let collection = spec
        .collections
        .iter()
        .find(|collection| collection.id == collection_id)
        .with_context(|| format!("source authority has no collection {collection_id}"))?;
    if !collection.data_types.iter().any(|data_type| data_type == "fastq") {
        bail!("selected source collection does not declare FASTQ data");
    }
    if collection.selected_accessions.is_empty() {
        bail!("selected source collection has no frozen run accessions");
    }
    if collection.sample_era.is_none() {
        bail!("selected source collection has no declared sample era");
    }
    Ok(collection)
}

fn records_by_accession(records: &[EnaRecord]) -> Result<BTreeMap<&str, &EnaRecord>> {
    let mut by_accession = BTreeMap::new();
    for record in records {
        let accession = required(&record.run_accession, "run accession")?;
        if by_accession.insert(accession, record).is_some() {
            bail!("ENA returned duplicate run metadata for {accession}");
        }
    }
    Ok(by_accession)
}

fn require_utc(value: &str) -> Result<()> {
    let retrieved_at = DateTime::<FixedOffset>::parse_from_rfc3339(value)
        .context("retrieval timestamp is not RFC 3339")?;
    if retrieved_at.offset().local_minus_utc() != 0 {
        bail!("retrieval timestamp must use UTC");
    }
    Ok(())
}

fn refuse_existing_output(path: &Path) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => bail!("refusing to replace acquisition plan: {}", path.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error)
            .with_context(|| format!("inspect acquisition plan output {}", path.display())),
    }
}

fn resolve_selected_collection_sample(
    collection: &BenchmarkDataSourceCollection,
    accession: &str,
    record: &EnaRecord,
    species: &str,
) -> Result<CorpusAcquisitionSample> {
    if required(&record.run_accession, "run accession")? != accession {
        bail!("ENA run identity disagrees for {accession}");
    }
    let layout = match required(&record.library_layout, "library layout")? {
        "SINGLE" => CorpusAcquisitionLayout::Se,
        "PAIRED" => CorpusAcquisitionLayout::Pe,
        value => bail!("ENA library layout {value} is unsupported for {accession}"),
    };
    if required(&record.scientific_name, "scientific name")? != species {
        bail!("ENA scientific name disagrees for {accession}");
    }
    let files = resolve_record_files(accession, layout, record)?;
    let total_size = files.iter().try_fold(0_u64, |total, file| {
        total.checked_add(file.size_bytes).context("sample FASTQ size overflows")
    })?;
    Ok(CorpusAcquisitionSample {
        accession: accession.to_string(),
        study_accession: required(&record.study_accession, "study accession")?.to_string(),
        sample_accession: required(&record.sample_accession, "sample accession")?.to_string(),
        experiment_accession: required(&record.experiment_accession, "experiment accession")?
            .to_string(),
        era: collection
            .sample_era
            .clone()
            .context("selected source collection has no declared sample era")?,
        layout,
        size_band: size_band(total_size).to_string(),
        selection_reason: collection.selection_scope.clone(),
        scientific_name: species.to_string(),
        library_source: required(&record.library_source, "library source")?.to_string(),
        library_strategy: required(&record.library_strategy, "library strategy")?.to_string(),
        library_selection: required(&record.library_selection, "library selection")?.to_string(),
        instrument_platform: required(&record.instrument_platform, "instrument platform")?
            .to_string(),
        instrument_model: required(&record.instrument_model, "instrument model")?.to_string(),
        archive_base_count: record
            .base_count
            .filter(|value| *value > 0)
            .context("ENA base count is missing or zero")?,
        archive_read_count: record
            .read_count
            .filter(|value| *value > 0)
            .context("ENA read count is missing or zero")?,
        first_public: required(&record.first_public, "first publication")?.to_string(),
        last_updated: required(&record.last_updated, "last update")?.to_string(),
        source_report_url: bijux_dna_db_ena::client::build_filereport_url(
            accession,
            EnaResultKind::ReadRun,
        ),
        files,
    })
}

fn size_band(size_bytes: u64) -> &'static str {
    match size_bytes {
        0..100_000_000 => "under_100mb",
        100_000_000..500_000_000 => "under_500mb",
        500_000_000..1_000_000_000 => "under_1000mb",
        _ => "at_least_1000mb",
    }
}

fn resolve_sample(
    spec: &FastqCorpusSourceSpec,
    sample: &FastqCorpusSourceSample,
    record: &EnaRecord,
) -> Result<CorpusAcquisitionSample> {
    let accession = required(&record.run_accession, "run accession")?;
    if accession != sample.accession
        || required(&record.study_accession, "study accession")? != sample.study_accession
    {
        bail!("ENA run or study identity disagrees for {}", sample.accession);
    }
    let expected_layout = match sample.layout {
        CorpusAcquisitionLayout::Se => "SINGLE",
        CorpusAcquisitionLayout::Pe => "PAIRED",
    };
    if required(&record.library_layout, "library layout")? != expected_layout {
        bail!("ENA library layout disagrees for {}", sample.accession);
    }
    if required(&record.scientific_name, "scientific name")? != spec.species {
        bail!("ENA species identity disagrees for {}", sample.accession);
    }
    let files = resolve_files(sample, record)?;
    Ok(CorpusAcquisitionSample {
        accession: sample.accession.clone(),
        study_accession: sample.study_accession.clone(),
        sample_accession: required(&record.sample_accession, "sample accession")?.to_string(),
        experiment_accession: required(&record.experiment_accession, "experiment accession")?
            .to_string(),
        era: sample.era.clone(),
        layout: sample.layout,
        size_band: sample.size_band.clone(),
        selection_reason: sample.reason.clone(),
        scientific_name: spec.species.clone(),
        library_source: required(&record.library_source, "library source")?.to_string(),
        library_strategy: required(&record.library_strategy, "library strategy")?.to_string(),
        library_selection: required(&record.library_selection, "library selection")?.to_string(),
        instrument_platform: required(&record.instrument_platform, "instrument platform")?
            .to_string(),
        instrument_model: required(&record.instrument_model, "instrument model")?.to_string(),
        archive_base_count: record
            .base_count
            .filter(|value| *value > 0)
            .context("ENA base count is missing or zero")?,
        archive_read_count: record
            .read_count
            .filter(|value| *value > 0)
            .context("ENA read count is missing or zero")?,
        first_public: required(&record.first_public, "first publication")?.to_string(),
        last_updated: required(&record.last_updated, "last update")?.to_string(),
        source_report_url: bijux_dna_db_ena::client::build_filereport_url(
            &sample.accession,
            EnaResultKind::ReadRun,
        ),
        files,
    })
}

fn resolve_files(
    sample: &FastqCorpusSourceSample,
    record: &EnaRecord,
) -> Result<Vec<CorpusAcquisitionFile>> {
    resolve_record_files(&sample.accession, sample.layout, record)
}

fn resolve_record_files(
    accession: &str,
    layout: CorpusAcquisitionLayout,
    record: &EnaRecord,
) -> Result<Vec<CorpusAcquisitionFile>> {
    let urls = record.preferred_urls(EnaFileSource::FastqFtp, EnaSourcePreference::Https);
    let file_count = urls.len();
    let layout_file_count_valid = match layout {
        CorpusAcquisitionLayout::Se => file_count == 1,
        CorpusAcquisitionLayout::Pe => matches!(file_count, 1 | 2),
    };
    if !layout_file_count_valid
        || record.fastq_md5.len() != file_count
        || record.fastq_bytes.len() != file_count
    {
        bail!("ENA FASTQ location, digest, size, or layout counts disagree for {}", accession);
    }
    urls.into_iter()
        .zip(&record.fastq_md5)
        .zip(&record.fastq_bytes)
        .enumerate()
        .map(|(index, ((source_url, source_md5), size_bytes))| {
            let name = source_url
                .rsplit('/')
                .next()
                .filter(|value| !value.is_empty())
                .context("ENA FASTQ URL has no file name")?;
            Ok(CorpusAcquisitionFile {
                mate: (layout == CorpusAcquisitionLayout::Pe && file_count == 2)
                    .then_some(u8::try_from(index + 1)?),
                name: name.to_string(),
                source_url,
                source_md5: source_md5.clone(),
                size_bytes: *size_bytes,
            })
        })
        .collect()
}

fn required<'a>(value: &'a Option<String>, label: &str) -> Result<&'a str> {
    value
        .as_deref()
        .filter(|value| !value.trim().is_empty() && *value == value.trim())
        .with_context(|| format!("ENA {label} is missing or malformed"))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use bijux_dna_db_ena::EnaRecord;

    use super::{
        derive_fastq_corpus_acquisition_plan, derive_selected_fastq_collection_acquisition_plan,
        load_fastq_corpus_source_spec,
    };

    #[test]
    fn checked_in_human_source_has_access_truth_and_projection_authority() -> anyhow::Result<()> {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| anyhow::anyhow!("benchmark crate is outside the workspace"))?;
        let source = load_fastq_corpus_source_spec(
            &repository_root.join("benchmarks/corpora/human-short-read-diversity.toml"),
        )?;

        assert_eq!(source.samples.len(), 20);
        assert_eq!(source.policy.projected_storage_bytes, 5_457_128_348);
        assert_eq!(source.policy.projected_cpu_hours, 240);
        assert!(source.policy.access_terms_url.contains("ebi.ac.uk/ena"));
        assert_eq!(source.policy.truth_strategy, "observational_only");
        Ok(())
    }

    #[test]
    fn checked_in_selected_fastq_collections_have_acquisition_context() -> anyhow::Result<()> {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| anyhow::anyhow!("benchmark crate is outside the workspace"))?;
        let mut selected_fastq_collections = 0_usize;
        for source_name in [
            "amplicon-community-challenge.toml",
            "ancient-dna-damage-controls.toml",
            "environmental-taxonomy-challenge.toml",
            "read-processing-challenge.toml",
        ] {
            let source = super::super::load_benchmark_data_source_spec(
                &repository_root.join("benchmarks/corpora").join(source_name),
            )?;
            for collection in source.collections.iter().filter(|collection| {
                !collection.selected_accessions.is_empty()
                    && collection.data_types.iter().any(|data_type| data_type == "fastq")
            }) {
                selected_fastq_collections += 1;
                assert!(matches!(
                    collection.sample_era.as_deref(),
                    Some("ancient" | "modern" | "not_applicable")
                ));
            }
        }
        assert_eq!(selected_fastq_collections, 9);
        Ok(())
    }

    #[test]
    fn offline_derivation_locks_urls_digests_sizes_and_context() -> anyhow::Result<()> {
        let temporary = tempfile::tempdir()?;
        let spec_path = temporary.path().join("corpus.toml");
        fs::write(
            &spec_path,
            concat!(
                "schema_version = \"bijux.benchmark.fastq_corpus_source.v1\"\n",
                "corpus_id = \"real-panel\"\n",
                "portfolio_id = \"real-panel-diversity\"\n",
                "class = \"production_real\"\n",
                "species = \"Homo sapiens\"\n",
                "description = \"real archive test panel\"\n",
                "source_authority = \"European Nucleotide Archive\"\n",
                "access_classification = \"public\"\n",
                "redistribution_classification = \"public-source-copy\"\n",
                "access_terms_url = \"https://www.ebi.ac.uk/ena/browser/about/policies\"\n",
                "attribution = \"cite source studies and accessions\"\n",
                "projected_storage_bytes = 1000\n",
                "projected_cpu_hours = 2\n",
                "projection_basis = \"bounded unit-test fixture projection\"\n",
                "truth_roles = [\"archive_context\"]\n",
                "truth_strategy = \"archive_metadata\"\n",
                "known_limitations = [\"archive metadata is observational\"]\n",
                "sample_context = \"Two archive test libraries\"\n",
                "assay_context = \"Contract-only short-read assay\"\n",
                "strata = [\"era\", \"layout\"]\n",
                "controls = [\"layout_contrast\"]\n",
                "held_out_split = \"No scientific split; fixture use only\"\n",
                "lineage = \"Archive fixture records to acquisition plan\"\n",
                "allowed_uses = [\"contract_validation\"]\n",
                "target_total = 2\n",
                "[[samples]]\n",
                "accession = \"ERR000001\"\n",
                "study_accession = \"PRJEB1\"\n",
                "era = \"ancient\"\n",
                "layout = \"se\"\n",
                "size_band = \"small\"\n",
                "reason = \"ancient single-end coverage\"\n",
                "[[samples]]\n",
                "accession = \"ERR000002\"\n",
                "study_accession = \"PRJEB1\"\n",
                "era = \"modern\"\n",
                "layout = \"pe\"\n",
                "size_band = \"medium\"\n",
                "reason = \"modern paired-end coverage\"\n",
            ),
        )?;
        let records = [
            record("ERR000001", "SINGLE", 'a', &[101]),
            record("ERR000002", "PAIRED", 'b', &[202, 303]),
        ];

        let plan =
            derive_fastq_corpus_acquisition_plan(&spec_path, &records, "2026-08-03T00:00:00Z")?;

        assert_eq!(plan.sample_count, 2);
        assert_eq!(plan.file_count, 3);
        assert_eq!(plan.total_size_bytes, 606);
        assert_eq!(plan.policy.projected_storage_bytes, 1000);
        assert_eq!(plan.policy.projected_cpu_hours, 2);
        assert_eq!(plan.policy.truth_strategy, "archive_metadata");
        assert_eq!(plan.samples[1].files[1].mate, Some(2));
        assert_eq!(plan.samples[1].files[1].source_md5, "b".repeat(32));
        assert_eq!(plan.samples[0].library_selection, "RANDOM");
        assert!(plan.samples[0].source_report_url.contains("fastq_md5"));
        Ok(())
    }

    #[test]
    fn offline_derivation_rejects_layout_disagreement() -> anyhow::Result<()> {
        let temporary = tempfile::tempdir()?;
        let spec_path = temporary.path().join("corpus.toml");
        fs::write(
            &spec_path,
            concat!(
                "schema_version = \"bijux.benchmark.fastq_corpus_source.v1\"\n",
                "corpus_id = \"real-panel\"\n",
                "portfolio_id = \"real-panel-diversity\"\n",
                "class = \"production_real\"\n",
                "species = \"Homo sapiens\"\n",
                "description = \"real archive test panel\"\n",
                "source_authority = \"European Nucleotide Archive\"\n",
                "access_classification = \"public\"\n",
                "redistribution_classification = \"public-source-copy\"\n",
                "access_terms_url = \"https://www.ebi.ac.uk/ena/browser/about/policies\"\n",
                "attribution = \"cite source studies and accessions\"\n",
                "projected_storage_bytes = 1000\n",
                "projected_cpu_hours = 1\n",
                "projection_basis = \"bounded unit-test fixture projection\"\n",
                "truth_roles = [\"archive_context\"]\n",
                "truth_strategy = \"archive_metadata\"\n",
                "known_limitations = [\"archive metadata is observational\"]\n",
                "sample_context = \"One archive test library\"\n",
                "assay_context = \"Contract-only short-read assay\"\n",
                "strata = [\"era\", \"layout\"]\n",
                "controls = [\"layout_validation\"]\n",
                "held_out_split = \"No scientific split; fixture use only\"\n",
                "lineage = \"Archive fixture record to acquisition rejection\"\n",
                "allowed_uses = [\"contract_validation\"]\n",
                "target_total = 1\n",
                "[[samples]]\n",
                "accession = \"ERR000001\"\n",
                "study_accession = \"PRJEB1\"\n",
                "era = \"ancient\"\n",
                "layout = \"se\"\n",
                "size_band = \"small\"\n",
                "reason = \"ancient single-end coverage\"\n",
            ),
        )?;

        let Err(error) = derive_fastq_corpus_acquisition_plan(
            &spec_path,
            &[record("ERR000001", "PAIRED", 'a', &[101, 102])],
            "2026-08-03T00:00:00Z",
        ) else {
            anyhow::bail!("layout disagreement must fail");
        };

        assert!(error.to_string().contains("library layout disagrees"));
        Ok(())
    }

    #[test]
    fn selected_collection_derivation_closes_generic_source_authority() -> anyhow::Result<()> {
        let temporary = tempfile::tempdir()?;
        let spec_path = temporary.path().join("source.toml");
        write_selected_collection_source(&spec_path)?;
        let records = [
            record("ERR000001", "SINGLE", 'a', &[99_000_000]),
            record("ERR000002", "PAIRED", 'b', &[600_000_000]),
        ];

        let plan = derive_selected_fastq_collection_acquisition_plan(
            &spec_path,
            "selected-runs",
            &records,
            "2026-08-16T00:00:00Z",
        )?;

        assert_eq!(plan.corpus_id, "stage-truth-panel-selected-runs");
        assert_eq!(plan.portfolio_id, "stage-truth-panel");
        assert_eq!(plan.species, "Homo sapiens");
        assert_eq!(plan.sample_count, 2);
        assert_eq!(plan.file_count, 2);
        assert_eq!(plan.total_size_bytes, 699_000_000);
        assert_eq!(plan.samples[0].era, "modern");
        assert_eq!(plan.samples[0].size_band, "under_100mb");
        assert_eq!(plan.samples[1].size_band, "under_1000mb");
        assert_eq!(plan.samples[1].files[0].mate, None);
        assert_eq!(plan.policy.truth_strategy, "orthogonal_expectations");
        Ok(())
    }

    #[test]
    fn selected_collection_derivation_rejects_mixed_species() -> anyhow::Result<()> {
        let temporary = tempfile::tempdir()?;
        let spec_path = temporary.path().join("source.toml");
        write_selected_collection_source(&spec_path)?;
        let first = record("ERR000001", "SINGLE", 'a', &[100]);
        let mut second = record("ERR000002", "PAIRED", 'b', &[200, 300]);
        second.scientific_name = Some("Mus musculus".to_string());

        let error = derive_selected_fastq_collection_acquisition_plan(
            &spec_path,
            "selected-runs",
            &[first, second],
            "2026-08-16T00:00:00Z",
        )
        .expect_err("mixed species must be split into separate acquisition collections");

        assert!(error.to_string().contains("more than one ENA scientific name"));
        Ok(())
    }

    fn write_selected_collection_source(path: &Path) -> anyhow::Result<()> {
        fs::write(
            path,
            concat!(
                "schema_version = \"bijux.benchmark.data_source.v1\"\n",
                "portfolio_id = \"stage-truth-panel\"\n",
                "class = \"production_real\"\n",
                "access_classification = \"public\"\n",
                "redistribution_classification = \"accession-referenced-source-copy\"\n",
                "access_terms_url = \"https://www.ebi.ac.uk/ena/browser/about/policies\"\n",
                "attribution = \"Cite source studies, accessions, and ENA.\"\n",
                "projected_storage_bytes = 1000000000\n",
                "projected_cpu_hours = 20\n",
                "projection_basis = \"Bounded selected-run acquisition and validation.\"\n",
                "truth_roles = [\"stage_truth\"]\n",
                "truth_strategy = \"orthogonal_expectations\"\n",
                "known_limitations = [\"Archive metadata is not exact biological truth.\"]\n",
                "description = \"Selected real runs for stage truth evaluation.\"\n",
                "sample_context = \"Two public biological libraries.\"\n",
                "assay_context = \"Archive FASTQ reads.\"\n",
                "strata = [\"source_collection\"]\n",
                "controls = [\"archive_metadata_closure\"]\n",
                "held_out_split = \"Keep the second run held out.\"\n",
                "lineage = \"Selected accessions to checksummed acquisition plan.\"\n",
                "allowed_uses = [\"performance\", \"robustness\"]\n",
                "data_types = [\"fastq\"]\n",
                "target_sample_count = 2\n",
                "[[collections]]\n",
                "id = \"selected-runs\"\n",
                "authority = \"European Nucleotide Archive\"\n",
                "accession = \"PRJEB1\"\n",
                "source_url = \"https://www.ebi.ac.uk/ena/browser/view/PRJEB1\"\n",
                "access_terms_url = \"https://www.ebi.ac.uk/ena/browser/about/policies\"\n",
                "data_types = [\"fastq\"]\n",
                "planned_sample_count = 2\n",
                "selected_accessions = [\"ERR000001\", \"ERR000002\"]\n",
                "sample_era = \"modern\"\n",
                "selection_scope = \"The two exact runs selected for contract closure.\"\n",
                "truth_basis = \"Archive identities and orthogonal expectations.\"\n",
                "known_limitations = [\"The runs do not supply exact per-read truth.\"]\n",
            ),
        )?;
        Ok(())
    }

    fn record(accession: &str, layout: &str, digest: char, sizes: &[u64]) -> EnaRecord {
        EnaRecord {
            study_accession: Some("PRJEB1".to_string()),
            sample_accession: Some(format!("SAMEA-{accession}")),
            experiment_accession: Some(format!("ERX-{accession}")),
            run_accession: Some(accession.to_string()),
            analysis_accession: None,
            tax_id: Some("9606".to_string()),
            scientific_name: Some("Homo sapiens".to_string()),
            library_layout: Some(layout.to_string()),
            library_source: Some("GENOMIC".to_string()),
            library_strategy: Some("WGS".to_string()),
            library_selection: Some("RANDOM".to_string()),
            instrument_platform: Some("ILLUMINA".to_string()),
            instrument_model: Some("Illumina HiSeq".to_string()),
            base_count: Some(1_000),
            read_count: Some(10),
            fastq_bytes: sizes.to_vec(),
            fastq_ftp: sizes
                .iter()
                .enumerate()
                .map(|(index, _)| format!("ftp.sra.ebi.ac.uk/{accession}_{}.fastq.gz", index + 1))
                .collect(),
            fastq_md5: sizes.iter().map(|_| digest.to_string().repeat(32)).collect(),
            first_public: Some("2020-01-01".to_string()),
            last_updated: Some("2020-01-02".to_string()),
            submitted_ftp: Vec::new(),
            sra_ftp: Vec::new(),
            bam_ftp: Vec::new(),
        }
    }
}
