//! Owner: bijux-dna-bench
//! Typed ENA metadata resolution and immutable corpus acquisition-plan publication.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_corpus_acquisition_plan, CorpusAcquisitionFile, CorpusAcquisitionLayout,
    CorpusAcquisitionPlan, CorpusAcquisitionSample, CORPUS_ACQUISITION_SCHEMA_V1,
};
use bijux_dna_db_ena::{
    EnaClient, EnaFileSource, EnaQuery, EnaRecord, EnaResultKind, EnaSourcePreference,
};
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

const CORPUS_SPEC_SCHEMA_V1: &str = "bijux.corpus_spec.v1";
const MAX_CORPUS_SPEC_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusAcquisitionRequest {
    pub spec_path: PathBuf,
    pub output_path: PathBuf,
    pub retrieved_at_utc: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusSpec {
    schema_version: String,
    corpus_id: String,
    portfolio_id: String,
    species: String,
    #[serde(default)]
    species_id: Option<String>,
    #[serde(default)]
    preferred_root: Option<String>,
    #[serde(default)]
    description: Option<String>,
    target_total: usize,
    #[serde(default)]
    target_ancient_se: Option<usize>,
    #[serde(default)]
    target_ancient_pe: Option<usize>,
    #[serde(default)]
    target_modern_se: Option<usize>,
    #[serde(default)]
    target_modern_pe: Option<usize>,
    samples: Vec<CorpusSpecSample>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusSpecSample {
    accession: String,
    study_accession: String,
    era: String,
    layout: CorpusAcquisitionLayout,
    size_band: String,
    reason: String,
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
    let spec = load_spec(&request.spec_path)?;
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

/// Derive a validated plan from caller-supplied ENA records for deterministic offline replay.
///
/// # Errors
/// Returns an error under the same identity and metadata conditions as live acquisition.
pub fn derive_fastq_corpus_acquisition_plan(
    spec_path: &Path,
    records: &[EnaRecord],
    retrieved_at_utc: &str,
) -> Result<CorpusAcquisitionPlan> {
    let spec = load_spec(spec_path)?;
    derive_plan(spec_path, &spec, records, retrieved_at_utc)
}

fn load_spec(path: &Path) -> Result<CorpusSpec> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_CORPUS_SPEC_BYTES)
        .with_context(|| format!("read corpus specification {}", path.display()))?;
    let spec: CorpusSpec =
        toml::from_str(std::str::from_utf8(&bytes).context("corpus specification is not UTF-8")?)
            .with_context(|| format!("parse corpus specification {}", path.display()))?;
    validate_spec(&spec)?;
    Ok(spec)
}

fn validate_spec(spec: &CorpusSpec) -> Result<()> {
    if spec.schema_version != CORPUS_SPEC_SCHEMA_V1 {
        bail!("corpus specification schema is unsupported");
    }
    if spec.samples.is_empty() || spec.samples.len() != spec.target_total {
        bail!("corpus specification target total disagrees with samples");
    }
    if !valid_identifier(&spec.corpus_id) || !valid_identifier(&spec.portfolio_id) {
        bail!("corpus specification identity is malformed");
    }
    if spec.species.trim().is_empty() || spec.species != spec.species.trim() {
        bail!("corpus specification species is empty or malformed");
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
        bail!("corpus specification cohort targets do not close over target total");
    }
    let mut accessions = BTreeSet::new();
    for sample in &spec.samples {
        if !accessions.insert(sample.accession.as_str()) {
            bail!("corpus specification repeats a run accession");
        }
        for (value, label) in [
            (&sample.accession, "run accession"),
            (&sample.study_accession, "study accession"),
            (&sample.era, "era"),
            (&sample.size_band, "size band"),
            (&sample.reason, "selection reason"),
        ] {
            if value.trim().is_empty() || value != value.trim() {
                bail!("corpus specification {label} is empty or malformed");
            }
        }
    }
    Ok(())
}

fn derive_plan(
    spec_path: &Path,
    spec: &CorpusSpec,
    records: &[EnaRecord],
    retrieved_at_utc: &str,
) -> Result<CorpusAcquisitionPlan> {
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
        schema_version: CORPUS_ACQUISITION_SCHEMA_V1.to_string(),
        corpus_id: spec.corpus_id.clone(),
        portfolio_id: spec.portfolio_id.clone(),
        species: spec.species.clone(),
        retrieved_at_utc: retrieved_at_utc.to_string(),
        source_authority: "European Nucleotide Archive".to_string(),
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

fn resolve_sample(
    spec: &CorpusSpec,
    sample: &CorpusSpecSample,
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
    sample: &CorpusSpecSample,
    record: &EnaRecord,
) -> Result<Vec<CorpusAcquisitionFile>> {
    let urls = record.preferred_urls(EnaFileSource::FastqFtp, EnaSourcePreference::Https);
    let expected = match sample.layout {
        CorpusAcquisitionLayout::Se => 1,
        CorpusAcquisitionLayout::Pe => 2,
    };
    if urls.len() != expected
        || record.fastq_md5.len() != expected
        || record.fastq_bytes.len() != expected
    {
        bail!(
            "ENA FASTQ location, digest, size, or layout counts disagree for {}",
            sample.accession
        );
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
                mate: (expected == 2).then_some(u8::try_from(index + 1)?),
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

fn valid_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use std::fs;

    use bijux_dna_db_ena::EnaRecord;

    use super::derive_fastq_corpus_acquisition_plan;

    #[test]
    fn offline_derivation_locks_urls_digests_sizes_and_context() -> anyhow::Result<()> {
        let temporary = tempfile::tempdir()?;
        let spec_path = temporary.path().join("corpus.toml");
        fs::write(
            &spec_path,
            concat!(
                "schema_version = \"bijux.corpus_spec.v1\"\n",
                "corpus_id = \"real-panel\"\n",
                "portfolio_id = \"real-panel-diversity\"\n",
                "species = \"Homo sapiens\"\n",
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
                "schema_version = \"bijux.corpus_spec.v1\"\n",
                "corpus_id = \"real-panel\"\n",
                "portfolio_id = \"real-panel-diversity\"\n",
                "species = \"Homo sapiens\"\n",
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
