use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_db_ena::client::EnaClient;
use bijux_dna_db_ena::download::{build_download_tasks, download_tasks, DownloadConfig};
use bijux_dna_db_ena::model::{
    EnaFileSource, EnaQuery, EnaRecord, EnaResultKind, EnaSourcePreference,
};
use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct CorpusManifest {
    #[serde(default = "default_manifest_schema")]
    schema_version: String,
    files: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct CuratedCorpusSpec {
    schema_version: String,
    corpus_id: String,
    species: String,
    species_id: String,
    preferred_root: Option<PathBuf>,
    description: String,
    #[serde(default)]
    target_total: Option<usize>,
    #[serde(default)]
    target_ancient_se: Option<usize>,
    #[serde(default)]
    target_ancient_pe: Option<usize>,
    #[serde(default)]
    target_modern_se: Option<usize>,
    #[serde(default)]
    target_modern_pe: Option<usize>,
    samples: Vec<CuratedSampleSpec>,
}

#[derive(Debug, Clone, Deserialize)]
struct CuratedSampleSpec {
    accession: String,
    study_accession: String,
    era: CorpusEra,
    layout: CorpusLayout,
    size_band: SizeBand,
    reason: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
enum CorpusEra {
    Ancient,
    Modern,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
enum CorpusLayout {
    Se,
    Pe,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum SizeBand {
    #[serde(rename = "under_100mb")]
    Under100,
    #[serde(rename = "under_500mb")]
    Under500,
    #[serde(rename = "under_1000mb")]
    Under1000,
}

#[derive(Debug, Serialize)]
struct CuratedCorpusSnapshot {
    schema_version: &'static str,
    corpus_id: String,
    species_id: String,
    species_display: String,
    description: String,
    selected: Vec<CuratedSelectionRow>,
    summary: CuratedCorpusSummary,
}

#[derive(Debug, Clone, Serialize)]
struct CuratedSelectionRow {
    accession: String,
    study_accession: String,
    sample_accession: Option<String>,
    scientific_name: String,
    era: &'static str,
    layout: &'static str,
    size_band: &'static str,
    library_source: String,
    library_strategy: String,
    instrument_model: String,
    fastq_bytes: Vec<u64>,
    total_fastq_bytes: u64,
    fastq_ftp: Vec<String>,
    reason: String,
}

#[derive(Debug, Serialize)]
struct CuratedCorpusSummary {
    samples_total: usize,
    ancient_se: usize,
    ancient_pe: usize,
    modern_se: usize,
    modern_pe: usize,
    under_100mb: usize,
    under_500mb: usize,
    under_1000mb: usize,
}

impl CorpusEra {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ancient => "ancient",
            Self::Modern => "modern",
        }
    }
}

impl CorpusLayout {
    fn as_str(self) -> &'static str {
        match self {
            Self::Se => "se",
            Self::Pe => "pe",
        }
    }
}

impl SizeBand {
    fn as_str(self) -> &'static str {
        match self {
            Self::Under100 => "under_100mb",
            Self::Under500 => "under_500mb",
            Self::Under1000 => "under_1000mb",
        }
    }

    fn upper_bound_bytes(self) -> u64 {
        match self {
            Self::Under100 => 100_000_000,
            Self::Under500 => 500_000_000,
            Self::Under1000 => 1_000_000_000,
        }
    }
}

#[derive(Debug, Serialize)]
struct CorpusListJson {
    schema_version: &'static str,
    corpora: Vec<CorpusInputs>,
}

#[derive(Debug, Serialize)]
struct CorpusInputs {
    corpus: String,
    files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ManifestDiffJson {
    schema_version: &'static str,
    left: String,
    right: String,
    added: Vec<String>,
    removed: Vec<String>,
    changed: Vec<String>,
}

type ManifestDiff = (String, String, Vec<String>, Vec<String>, Vec<String>);

#[derive(Debug, Clone)]
struct SampleFiles {
    r1: Option<PathBuf>,
    r2: Option<PathBuf>,
}

fn default_manifest_schema() -> String {
    "bijux.corpus_manifest.v1".to_string()
}

/// # Errors
/// Returns an error if the curated corpus spec is invalid, ENA metadata does
/// not satisfy the declared selection contract, or downloads/normalization
/// fail.
pub fn materialize_corpus(
    cwd: &Path,
    args: &crate::commands::cli::CorpusMaterializeArgs,
) -> Result<()> {
    let spec_path = resolve_path(cwd, &args.spec);
    let spec = load_curated_spec(&spec_path)?;
    validate_curated_spec(&spec)?;
    let root = args
        .root
        .clone()
        .or_else(|| spec.preferred_root.clone())
        .ok_or_else(|| anyhow!("curated corpus spec requires --root or preferred_root"))?;
    let root = resolve_path(cwd, &root);
    let records = fetch_curated_records(&spec)?;
    let curated_rows = build_curated_rows(&spec, &records)?;

    if args.dry_run {
        let summary = summarize_curated_rows(&curated_rows);
        println!("corpus_id={}", spec.corpus_id);
        println!("root={}", root.display());
        println!("samples_total={}", summary.samples_total);
        println!("ancient_se={}", summary.ancient_se);
        println!("ancient_pe={}", summary.ancient_pe);
        println!("modern_se={}", summary.modern_se);
        println!("modern_pe={}", summary.modern_pe);
        return Ok(());
    }

    bijux_dna_infra::ensure_dir(&root).with_context(|| format!("create {}", root.display()))?;
    let raw_dir = root.join("raw");
    bijux_dna_infra::ensure_dir(&raw_dir)
        .with_context(|| format!("create {}", raw_dir.display()))?;

    let dl_cfg = DownloadConfig {
        output_dir: raw_dir.clone(),
        jobs: args.jobs,
        retries: args.retries,
        source: EnaFileSource::FastqFtp,
        preference: EnaSourcePreference::Https,
        dry_run: false,
    };
    let tasks = build_download_tasks(&records, &dl_cfg);
    ensure_only_expected_raw_files(&raw_dir, &tasks)?;
    let report = download_tasks(&tasks, &dl_cfg).context("download curated corpus FASTQ files")?;
    if report.failed > 0 {
        return Err(anyhow!(
            "curated corpus download failures: {} files failed",
            report.failed
        ));
    }

    let spec_copy = root.join("CORPUS_SPEC.toml");
    fs::copy(&spec_path, &spec_copy)
        .with_context(|| format!("copy {} -> {}", spec_path.display(), spec_copy.display()))?;
    write_curated_snapshot(&root, &spec, &curated_rows)?;
    set_fastq_readonly(&raw_dir)?;

    let corpus_arg = root.display().to_string();
    normalize_corpus(cwd, &corpus_arg)?;
    validate_corpus(cwd, &corpus_arg)?;

    println!("corpus_id={}", spec.corpus_id);
    println!("root={}", root.display());
    println!("downloaded={}", report.downloaded);
    println!(
        "snapshot={}",
        root.join("ENA_METADATA.snapshot.json").display()
    );
    println!("manifest={}", root.join("MANIFEST.json").display());
    Ok(())
}

/// # Errors
/// Returns an error if raw layout is missing or normalized copies cannot be written.
pub fn normalize_corpus(cwd: &Path, corpus: &str) -> Result<()> {
    let root = resolve_corpus_root(cwd, corpus);
    let raw = root.join("raw");
    if !raw.exists() {
        return Err(anyhow!("missing raw corpus directory {}", raw.display()));
    }
    let normalized = root.join("normalized");
    fs::create_dir_all(&normalized).with_context(|| format!("create {}", normalized.display()))?;
    clear_fastq_dir(&normalized)?;

    let mut keys = collect_sample_keys(&raw)?;
    keys.sort_by(|a, b| {
        let a_key =
            a.r1.as_ref()
                .or(a.r2.as_ref())
                .map_or_else(String::new, |p| p.to_string_lossy().to_string());
        let b_key =
            b.r1.as_ref()
                .or(b.r2.as_ref())
                .map_or_else(String::new, |p| p.to_string_lossy().to_string());
        a_key.cmp(&b_key)
    });
    if keys.is_empty() {
        return Err(anyhow!("no FASTQ files found under {}", raw.display()));
    }

    for (index, key) in keys.iter().enumerate() {
        let sample_id = format!("{:04}", index + 1);
        if let Some(r1) = &key.r1 {
            let dst = normalized.join(format!("sample_{sample_id}_R1.fastq.gz"));
            fs::copy(r1, &dst)
                .with_context(|| format!("copy {} -> {}", r1.display(), dst.display()))?;
        }
        if let Some(r2) = &key.r2 {
            let dst = normalized.join(format!("sample_{sample_id}_R2.fastq.gz"));
            fs::copy(r2, &dst)
                .with_context(|| format!("copy {} -> {}", r2.display(), dst.display()))?;
        }
    }

    write_manifest(&root)?;
    println!("normalized={}", normalized.display());
    Ok(())
}

/// # Errors
/// Returns an error if corpus layout/contracts/checksums are invalid.
pub fn validate_corpus(cwd: &Path, corpus: &str) -> Result<()> {
    let root = resolve_corpus_root(cwd, corpus);
    let raw = root.join("raw");
    let normalized = root.join("normalized");
    let manifest_path = root.join("MANIFEST.json");
    if !raw.exists() || !normalized.exists() || !manifest_path.exists() {
        return Err(anyhow!(
            "missing required corpus layout under {} (expected raw/, normalized/, MANIFEST.json)",
            root.display()
        ));
    }
    let raw_manifest = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let manifest: CorpusManifest = serde_json::from_str(&raw_manifest)
        .with_context(|| format!("parse {}", manifest_path.display()))?;
    if manifest.files.is_empty() {
        return Err(anyhow!("MANIFEST.json has empty files map"));
    }

    for (rel, expected) in &manifest.files {
        let path = root.join(rel);
        if !path.exists() {
            return Err(anyhow!("manifest entry missing file: {}", path.display()));
        }
        if !path
            .file_name()
            .and_then(|v| v.to_str())
            .is_some_and(|v| v.ends_with(".fastq.gz"))
        {
            continue;
        }
        let actual = bijux_dna_infra::hash_file_sha256(&path)
            .with_context(|| format!("hash {}", path.display()))?;
        if &actual != expected {
            return Err(anyhow!(
                "checksum mismatch for {} expected {} got {}",
                path.display(),
                expected,
                actual
            ));
        }
        ensure_gzip_integrity(&path)?;
    }

    let sample_to_reads = collect_sample_keys(&normalized)?;
    if sample_to_reads.is_empty() {
        return Err(anyhow!("no normalized sample_*.fastq.gz files found"));
    }
    for (idx, sample) in sample_to_reads.iter().enumerate() {
        let sample_name = format!("sample_{:04}", idx + 1);
        let Some(r1_path) = sample.r1.as_ref() else {
            return Err(anyhow!("{sample_name} missing R1"));
        };
        let h1 = first_fastq_header(r1_path)?;
        if h1.trim().is_empty() || !h1.starts_with('@') {
            return Err(anyhow!("{sample_name} has invalid R1 header"));
        }
        if let Some(r2_path) = sample.r2.as_ref() {
            let h2 = first_fastq_header(r2_path)?;
            let n1 = normalize_read_header(&h1);
            let n2 = normalize_read_header(&h2);
            if n1 != n2 {
                return Err(anyhow!(
                    "{sample_name} paired read-name mismatch: `{n1}` vs `{n2}`"
                ));
            }
        }
    }
    println!("corpus validation ok: {}", root.display());
    Ok(())
}

/// # Errors
/// Returns an error if corpus enumeration fails.
pub fn list_corpus_json(cwd: &Path, corpus: Option<&str>) -> Result<()> {
    let payload = CorpusListJson {
        schema_version: "bijux.corpus.list.v1",
        corpora: list_inputs(cwd, corpus)?,
    };
    crate::commands::cli::render::json::print_pretty(&payload)
}

/// # Errors
/// Returns an error if corpus enumeration fails.
pub fn list_corpus_text(cwd: &Path, corpus: Option<&str>) -> Result<()> {
    let rows = list_inputs(cwd, corpus)?;
    for corpus in rows {
        println!("{}:", corpus.corpus);
        for file in corpus.files {
            println!("  {file}");
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error if manifest comparison fails.
pub fn diff_manifests_json(cwd: &Path, left: &str, right: &str) -> Result<()> {
    let (left_name, right_name, added, removed, changed) = diff_manifests(cwd, left, right)?;
    let payload = ManifestDiffJson {
        schema_version: "bijux.corpus.diff.v1",
        left: left_name,
        right: right_name,
        added,
        removed,
        changed,
    };
    crate::commands::cli::render::json::print_pretty(&payload)
}

/// # Errors
/// Returns an error if manifest comparison fails.
pub fn diff_manifests_text(cwd: &Path, left: &str, right: &str) -> Result<()> {
    let (left_name, right_name, added, removed, changed) = diff_manifests(cwd, left, right)?;
    println!("left={left_name}");
    println!("right={right_name}");
    println!("added={}", added.len());
    for item in added {
        println!("  + {item}");
    }
    println!("removed={}", removed.len());
    for item in removed {
        println!("  - {item}");
    }
    println!("changed={}", changed.len());
    for item in changed {
        println!("  * {item}");
    }
    Ok(())
}

fn load_curated_spec(path: &Path) -> Result<CuratedCorpusSpec> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&crate::commands::benchmark_workspace::expand_env_placeholders(&raw)?)
        .with_context(|| format!("parse {}", path.display()))
}

fn validate_curated_spec(spec: &CuratedCorpusSpec) -> Result<()> {
    if spec.schema_version != "bijux.corpus_spec.v1" {
        return Err(anyhow!(
            "unsupported curated corpus schema `{}`",
            spec.schema_version
        ));
    }
    if spec.corpus_id.trim().is_empty() {
        return Err(anyhow!("curated corpus spec missing corpus_id"));
    }
    if spec.species.trim().is_empty() || spec.species_id.trim().is_empty() {
        return Err(anyhow!("curated corpus spec missing species identity"));
    }
    if normalize_species_id(&spec.species)? != spec.species_id {
        return Err(anyhow!(
            "species/species_id mismatch: `{}` does not normalize to `{}`",
            spec.species,
            spec.species_id
        ));
    }
    if spec.description.trim().is_empty() {
        return Err(anyhow!("curated corpus spec missing description"));
    }
    if spec.samples.is_empty() {
        return Err(anyhow!("curated corpus spec has zero samples"));
    }

    let mut accessions = BTreeSet::new();
    let mut ancient_single_end_count = 0usize;
    let mut ancient_paired_end_count = 0usize;
    let mut modern_single_end_count = 0usize;
    let mut modern_paired_end_count = 0usize;
    for sample in &spec.samples {
        if !accessions.insert(sample.accession.clone()) {
            return Err(anyhow!(
                "curated corpus spec repeats accession `{}`",
                sample.accession
            ));
        }
        if sample.study_accession.trim().is_empty() {
            return Err(anyhow!(
                "sample `{}` missing study_accession",
                sample.accession
            ));
        }
        if sample.reason.trim().is_empty() {
            return Err(anyhow!("sample `{}` missing reason", sample.accession));
        }
        match (sample.era, sample.layout) {
            (CorpusEra::Ancient, CorpusLayout::Se) => ancient_single_end_count += 1,
            (CorpusEra::Ancient, CorpusLayout::Pe) => ancient_paired_end_count += 1,
            (CorpusEra::Modern, CorpusLayout::Se) => modern_single_end_count += 1,
            (CorpusEra::Modern, CorpusLayout::Pe) => modern_paired_end_count += 1,
        }
    }
    if let Some(expected) = spec.target_total {
        if spec.samples.len() != expected {
            return Err(anyhow!(
                "curated corpus spec expected {expected} samples, found {}",
                spec.samples.len()
            ));
        }
    }
    ensure_expected_count(
        "ancient_se",
        spec.target_ancient_se,
        ancient_single_end_count,
    )?;
    ensure_expected_count(
        "ancient_pe",
        spec.target_ancient_pe,
        ancient_paired_end_count,
    )?;
    ensure_expected_count("modern_se", spec.target_modern_se, modern_single_end_count)?;
    ensure_expected_count("modern_pe", spec.target_modern_pe, modern_paired_end_count)?;
    Ok(())
}

fn ensure_expected_count(label: &str, expected: Option<usize>, actual: usize) -> Result<()> {
    if let Some(expected) = expected {
        if actual != expected {
            return Err(anyhow!(
                "curated corpus spec expected {expected} {label} entries, found {actual}"
            ));
        }
    }
    Ok(())
}

fn normalize_species_id(value: &str) -> Result<String> {
    let tokens = value
        .split_whitespace()
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    if tokens.len() != 2
        || tokens
            .iter()
            .any(|token| !token.chars().all(|ch| ch.is_ascii_alphabetic()))
    {
        return Err(anyhow!(
            "species must be a latin binomial like `Homo sapiens`, got `{value}`"
        ));
    }
    Ok(format!(
        "{}_{}",
        tokens[0].to_ascii_lowercase(),
        tokens[1].to_ascii_lowercase()
    ))
}

fn fetch_curated_records(spec: &CuratedCorpusSpec) -> Result<Vec<EnaRecord>> {
    let accessions = spec
        .samples
        .iter()
        .map(|sample| sample.accession.clone())
        .collect::<Vec<_>>();
    let query = EnaQuery {
        projects: Vec::new(),
        samples: Vec::new(),
        extra_accessions: accessions,
        result: EnaResultKind::ReadRun,
    };
    let client = EnaClient::new("bijux-dna/corpus-materialize").context("create ENA client")?;
    client
        .fetch_records(&query)
        .context("fetch curated ENA records")
}

fn build_curated_rows(
    spec: &CuratedCorpusSpec,
    records: &[EnaRecord],
) -> Result<Vec<CuratedSelectionRow>> {
    let record_map = records
        .iter()
        .map(|record| (record.accession_label(), record))
        .collect::<HashMap<_, _>>();
    let mut rows = Vec::with_capacity(spec.samples.len());
    for sample in &spec.samples {
        let record = record_map.get(&sample.accession).copied().ok_or_else(|| {
            anyhow!(
                "curated corpus accession `{}` missing from ENA response",
                sample.accession
            )
        })?;
        validate_curated_record(spec, sample, record)?;
        rows.push(CuratedSelectionRow {
            accession: sample.accession.clone(),
            study_accession: sample.study_accession.clone(),
            sample_accession: record.sample_accession.clone(),
            scientific_name: record.scientific_name.clone().ok_or_else(|| {
                anyhow!("accession `{}` missing scientific_name", sample.accession)
            })?,
            era: sample.era.as_str(),
            layout: sample.layout.as_str(),
            size_band: sample.size_band.as_str(),
            library_source: record.library_source.clone().ok_or_else(|| {
                anyhow!("accession `{}` missing library_source", sample.accession)
            })?,
            library_strategy: record.library_strategy.clone().ok_or_else(|| {
                anyhow!("accession `{}` missing library_strategy", sample.accession)
            })?,
            instrument_model: record.instrument_model.clone().ok_or_else(|| {
                anyhow!("accession `{}` missing instrument_model", sample.accession)
            })?,
            fastq_bytes: record.fastq_bytes.clone(),
            total_fastq_bytes: record.fastq_bytes.iter().sum(),
            fastq_ftp: record.fastq_ftp.clone(),
            reason: sample.reason.clone(),
        });
    }
    Ok(rows)
}

fn validate_curated_record(
    spec: &CuratedCorpusSpec,
    sample: &CuratedSampleSpec,
    record: &EnaRecord,
) -> Result<()> {
    if record.study_accession.as_deref() != Some(sample.study_accession.as_str()) {
        return Err(anyhow!(
            "accession `{}` expected study `{}`, got `{:?}`",
            sample.accession,
            sample.study_accession,
            record.study_accession
        ));
    }
    if record.scientific_name.as_deref() != Some(spec.species.as_str()) {
        return Err(anyhow!(
            "accession `{}` expected species `{}`, got `{:?}`",
            sample.accession,
            spec.species,
            record.scientific_name
        ));
    }
    if record.fastq_ftp.is_empty() {
        return Err(anyhow!(
            "accession `{}` missing fastq_ftp",
            sample.accession
        ));
    }
    if record.base_count.unwrap_or(0) == 0 || record.read_count.unwrap_or(0) == 0 {
        return Err(anyhow!(
            "accession `{}` missing base_count/read_count",
            sample.accession
        ));
    }
    if !has_declared_text(record.library_source.as_deref()) {
        return Err(anyhow!(
            "accession `{}` missing library_source",
            sample.accession
        ));
    }
    if !has_declared_text(record.library_strategy.as_deref()) {
        return Err(anyhow!(
            "accession `{}` missing library_strategy",
            sample.accession
        ));
    }
    if !has_declared_text(record.instrument_model.as_deref()) {
        return Err(anyhow!(
            "accession `{}` missing instrument_model",
            sample.accession
        ));
    }
    let actual_layout = infer_layout(record).ok_or_else(|| {
        anyhow!(
            "accession `{}` has unrecognized read layout",
            sample.accession
        )
    })?;
    if actual_layout != sample.layout {
        return Err(anyhow!(
            "accession `{}` expected layout `{}`, got `{}`",
            sample.accession,
            sample.layout.as_str(),
            actual_layout.as_str()
        ));
    }
    if sample.size_band.upper_bound_bytes() < record.fastq_bytes.iter().sum::<u64>() {
        return Err(anyhow!(
            "accession `{}` exceeds declared size band `{}`",
            sample.accession,
            sample.size_band.as_str()
        ));
    }
    Ok(())
}

fn infer_layout(record: &EnaRecord) -> Option<CorpusLayout> {
    if let Some(value) = record.library_layout.as_deref() {
        let normalized = value.trim().to_ascii_uppercase();
        if normalized == "SINGLE" {
            return Some(CorpusLayout::Se);
        }
        if normalized == "PAIRED" {
            return Some(CorpusLayout::Pe);
        }
    }
    match record.fastq_ftp.len() {
        1 => Some(CorpusLayout::Se),
        2 => Some(CorpusLayout::Pe),
        _ => None,
    }
}

fn summarize_curated_rows(rows: &[CuratedSelectionRow]) -> CuratedCorpusSummary {
    let mut summary = CuratedCorpusSummary {
        samples_total: rows.len(),
        ancient_se: 0,
        ancient_pe: 0,
        modern_se: 0,
        modern_pe: 0,
        under_100mb: 0,
        under_500mb: 0,
        under_1000mb: 0,
    };
    for row in rows {
        match (row.era, row.layout) {
            ("ancient", "se") => summary.ancient_se += 1,
            ("ancient", "pe") => summary.ancient_pe += 1,
            ("modern", "se") => summary.modern_se += 1,
            ("modern", "pe") => summary.modern_pe += 1,
            _ => {}
        }
        match row.size_band {
            "under_100mb" => summary.under_100mb += 1,
            "under_500mb" => summary.under_500mb += 1,
            "under_1000mb" => summary.under_1000mb += 1,
            _ => {}
        }
    }
    summary
}

fn ensure_only_expected_raw_files(
    raw_dir: &Path,
    tasks: &[bijux_dna_db_ena::download::DownloadTask],
) -> Result<()> {
    if !raw_dir.exists() {
        return Ok(());
    }
    let expected = tasks
        .iter()
        .map(|task| task.output.clone())
        .collect::<BTreeSet<_>>();
    let mut existing = Vec::new();
    collect_fastqs_recursive(raw_dir, &mut existing)?;
    let unexpected = existing
        .into_iter()
        .filter(|path| !expected.contains(path))
        .collect::<Vec<_>>();
    if unexpected.is_empty() {
        return Ok(());
    }
    let rendered = unexpected
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Err(anyhow!(
        "raw corpus directory contains unexpected FASTQ files: {rendered}"
    ))
}

fn write_curated_snapshot(
    root: &Path,
    spec: &CuratedCorpusSpec,
    rows: &[CuratedSelectionRow],
) -> Result<()> {
    let snapshot = CuratedCorpusSnapshot {
        schema_version: "bijux.ena_metadata_snapshot.v4",
        corpus_id: spec.corpus_id.clone(),
        species_id: spec.species_id.clone(),
        species_display: spec.species.clone(),
        description: spec.description.clone(),
        selected: rows.to_vec(),
        summary: summarize_curated_rows(rows),
    };
    let path = root.join("ENA_METADATA.snapshot.json");
    bijux_dna_infra::atomic_write_json(&path, &snapshot)
        .with_context(|| format!("write {}", path.display()))
}

fn list_inputs(cwd: &Path, corpus: Option<&str>) -> Result<Vec<CorpusInputs>> {
    let mut corpora = Vec::new();
    if let Some(value) = corpus {
        let root = resolve_corpus_root(cwd, value);
        corpora.push(corpus_inputs_for_root(cwd, &root)?);
        return Ok(corpora);
    }

    let data_root = cwd.join("examples").join("bijux-dna-data");
    if data_root.exists() {
        for entry in
            fs::read_dir(&data_root).with_context(|| format!("read {}", data_root.display()))?
        {
            let path = entry?.path();
            if !path.is_dir() {
                continue;
            }
            if path.join("normalized").exists() {
                corpora.push(corpus_inputs_for_root(cwd, &path)?);
            }
        }
    }
    corpora.sort_by(|a, b| a.corpus.cmp(&b.corpus));
    Ok(corpora)
}

fn corpus_inputs_for_root(cwd: &Path, root: &Path) -> Result<CorpusInputs> {
    let name = root
        .file_name()
        .and_then(|v| v.to_str())
        .filter(|value| !value.trim().is_empty())
        .map_or_else(|| root.display().to_string(), ToOwned::to_owned);
    let normalized = root.join("normalized");
    let mut files = if normalized.exists() {
        fs::read_dir(&normalized)
            .with_context(|| format!("read {}", normalized.display()))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                path.file_name()
                    .and_then(|v| v.to_str())
                    .is_some_and(|name| name.ends_with(".fastq.gz"))
            })
            .filter_map(|path| {
                path.strip_prefix(cwd)
                    .ok()
                    .map(|rel| rel.to_string_lossy().to_string())
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    files.sort();
    files.dedup();
    Ok(CorpusInputs {
        corpus: name,
        files,
    })
}

fn has_declared_text(value: Option<&str>) -> bool {
    value.is_some_and(|entry| !entry.trim().is_empty())
}

fn diff_manifests(cwd: &Path, left: &str, right: &str) -> Result<ManifestDiff> {
    let left_root = resolve_corpus_root(cwd, left);
    let right_root = resolve_corpus_root(cwd, right);
    let left_manifest = read_manifest(&left_root)?;
    let right_manifest = read_manifest(&right_root)?;
    let left_name = left_root.display().to_string();
    let right_name = right_root.display().to_string();

    let left_keys = left_manifest.files.keys().cloned().collect::<BTreeSet<_>>();
    let right_keys = right_manifest
        .files
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let added = right_keys
        .difference(&left_keys)
        .cloned()
        .collect::<Vec<_>>();
    let removed = left_keys
        .difference(&right_keys)
        .cloned()
        .collect::<Vec<_>>();
    let changed = left_keys
        .intersection(&right_keys)
        .filter(|key| left_manifest.files.get(*key) != right_manifest.files.get(*key))
        .cloned()
        .collect::<Vec<_>>();
    Ok((left_name, right_name, added, removed, changed))
}

fn read_manifest(root: &Path) -> Result<CorpusManifest> {
    let path = root.join("MANIFEST.json");
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn clear_fastq_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).with_context(|| format!("remove {}", path.display()))?;
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name.ends_with(".fastq.gz") {
            fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
        }
    }
    Ok(())
}

fn set_fastq_readonly(root: &Path) -> Result<()> {
    let mut files = Vec::new();
    collect_fastqs_recursive(root, &mut files)?;
    for path in files {
        let mut perms = fs::metadata(&path)
            .with_context(|| format!("stat {}", path.display()))?
            .permissions();
        perms.set_readonly(true);
        fs::set_permissions(&path, perms)
            .with_context(|| format!("chmod readonly {}", path.display()))?;
    }
    Ok(())
}

fn collect_sample_keys(root: &Path) -> Result<Vec<SampleFiles>> {
    let mut groups = BTreeMap::<String, SampleFiles>::new();
    let mut files = Vec::new();
    collect_fastqs_recursive(root, &mut files)?;
    for path in files {
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !name.ends_with(".fastq.gz") {
            continue;
        }
        let stem = name.trim_end_matches(".fastq.gz");
        let (key, read) = infer_key_and_read(stem);
        let group = groups
            .entry(key)
            .or_insert(SampleFiles { r1: None, r2: None });
        match read {
            ReadKind::R2 => group.r2 = Some(path),
            ReadKind::R1 | ReadKind::Single => group.r1 = Some(path),
        }
    }
    Ok(groups.into_values().collect())
}

fn collect_fastqs_recursive(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path
                .file_name()
                .and_then(|v| v.to_str())
                .is_some_and(|v| v.ends_with(".fastq.gz"))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(())
}

enum ReadKind {
    R1,
    R2,
    Single,
}

fn infer_key_and_read(stem: &str) -> (String, ReadKind) {
    for suffix in ["_R1", "_1", ".R1", ".1"] {
        if let Some(base) = stem.strip_suffix(suffix) {
            return (base.to_string(), ReadKind::R1);
        }
    }
    for suffix in ["_R2", "_2", ".R2", ".2"] {
        if let Some(base) = stem.strip_suffix(suffix) {
            return (base.to_string(), ReadKind::R2);
        }
    }
    (stem.to_string(), ReadKind::Single)
}

fn write_manifest(corpus_root: &Path) -> Result<()> {
    let mut files = BTreeMap::new();
    let mut paths = Vec::new();
    collect_fastqs_recursive(corpus_root, &mut paths)?;
    for path in paths {
        let rel = path.strip_prefix(corpus_root).map_or_else(
            |_| path.display().to_string(),
            |v| v.to_string_lossy().to_string(),
        );
        let digest = bijux_dna_infra::hash_file_sha256(&path)
            .with_context(|| format!("hash {}", path.display()))?;
        files.insert(rel, digest);
    }
    let manifest = CorpusManifest {
        schema_version: default_manifest_schema(),
        files,
    };
    let path = corpus_root.join("MANIFEST.json");
    bijux_dna_infra::atomic_write_json(&path, &manifest)
        .with_context(|| format!("write {}", path.display()))
}

fn resolve_corpus_root(cwd: &Path, corpus: &str) -> PathBuf {
    let raw = PathBuf::from(corpus);
    if raw.is_absolute() {
        return raw;
    }
    if corpus.contains('/') {
        return cwd.join(corpus);
    }
    cwd.join("examples").join("bijux-dna-data").join(corpus)
}

fn resolve_path(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn first_fastq_header(path: &Path) -> Result<String> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut reader = BufReader::new(MultiGzDecoder::new(file));
    let mut line = String::new();
    let _ = reader
        .read_line(&mut line)
        .with_context(|| format!("read {}", path.display()))?;
    Ok(line.trim().to_string())
}

fn ensure_gzip_integrity(path: &Path) -> Result<()> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut decoder = MultiGzDecoder::new(file);
    let mut sink = [0_u8; 16 * 1024];
    loop {
        let bytes = decoder
            .read(&mut sink)
            .with_context(|| format!("read gzip {}", path.display()))?;
        if bytes == 0 {
            break;
        }
    }
    Ok(())
}

fn normalize_read_header(header: &str) -> String {
    let raw = header.trim_start_matches('@');
    raw.trim_end_matches("/1")
        .trim_end_matches("/2")
        .split_whitespace()
        .next()
        .unwrap_or(raw)
        .to_string()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn sample_spec(
        accession: &str,
        study_accession: &str,
        era: CorpusEra,
        layout: CorpusLayout,
    ) -> CuratedSampleSpec {
        CuratedSampleSpec {
            accession: accession.to_string(),
            study_accession: study_accession.to_string(),
            era,
            layout,
            size_band: SizeBand::Under100,
            reason: "fixture".to_string(),
        }
    }

    #[test]
    fn curated_spec_enforces_declared_balance() {
        let spec = CuratedCorpusSpec {
            schema_version: "bijux.corpus_spec.v1".to_string(),
            corpus_id: "corpus-01".to_string(),
            species: "Homo sapiens".to_string(),
            species_id: "homo_sapiens".to_string(),
            preferred_root: None,
            description: "fixture".to_string(),
            target_total: Some(4),
            target_ancient_se: Some(1),
            target_ancient_pe: Some(1),
            target_modern_se: Some(1),
            target_modern_pe: Some(1),
            samples: vec![
                sample_spec("ERR1", "PRJEB1", CorpusEra::Ancient, CorpusLayout::Se),
                sample_spec("ERR2", "PRJEB1", CorpusEra::Ancient, CorpusLayout::Pe),
                sample_spec("ERR3", "PRJEB2", CorpusEra::Modern, CorpusLayout::Se),
                sample_spec("ERR4", "PRJEB2", CorpusEra::Modern, CorpusLayout::Pe),
            ],
        };
        validate_curated_spec(&spec).expect("spec should validate");
    }

    #[test]
    fn curated_spec_rejects_wrong_species_id() {
        let spec = CuratedCorpusSpec {
            schema_version: "bijux.corpus_spec.v1".to_string(),
            corpus_id: "corpus-01".to_string(),
            species: "Homo sapiens".to_string(),
            species_id: "human".to_string(),
            preferred_root: None,
            description: "fixture".to_string(),
            target_total: Some(1),
            target_ancient_se: Some(1),
            target_ancient_pe: Some(0),
            target_modern_se: Some(0),
            target_modern_pe: Some(0),
            samples: vec![sample_spec(
                "ERR1",
                "PRJEB1",
                CorpusEra::Ancient,
                CorpusLayout::Se,
            )],
        };
        let err = validate_curated_spec(&spec).expect_err("species id should be rejected");
        assert!(err.to_string().contains("species/species_id mismatch"));
    }

    #[test]
    fn size_band_upper_bounds_are_stable() {
        assert_eq!(SizeBand::Under100.upper_bound_bytes(), 100_000_000);
        assert_eq!(SizeBand::Under500.upper_bound_bytes(), 500_000_000);
        assert_eq!(SizeBand::Under1000.upper_bound_bytes(), 1_000_000_000);
    }
}
