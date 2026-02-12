use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_db_ena::client::EnaClient;
use bijux_dna_db_ena::download::{build_download_tasks, download_tasks, DownloadConfig};
use bijux_dna_db_ena::model::{
    EnaFileSource, EnaQuery, EnaRecord, EnaResultKind, EnaRunManifest, EnaSourcePreference,
};
use serde::{Deserialize, Serialize};

use crate::commands::cli::{EnaFetchArgs, EnaSelectArgs};

const MIN_FASTQ_BYTES: u64 = 1_000_000;
const MAX_FASTQ_BYTES: u64 = 200_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutKind {
    Se,
    Pe,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataSnapshot {
    schema_version: String,
    species_id: String,
    species_display: String,
    project: String,
    corpus_id: String,
    target_se: usize,
    target_pe: usize,
    selected: Vec<SelectionRow>,
    rejected: Vec<SelectionRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRow {
    accession: String,
    sample_accession: Option<String>,
    read_layout: String,
    library_type: String,
    instrument: String,
    base_count: u64,
    read_count: u64,
    fastq_ftp: Vec<String>,
    fastq_bytes: Vec<u64>,
    reason: String,
}

#[derive(Debug, Serialize)]
struct CorpusManifest {
    schema_version: &'static str,
    species_id: String,
    species_display: String,
    project: String,
    corpus_id: String,
    files: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct SpeciesIdentity {
    species_id: String,
    species_display: String,
}

/// # Errors
/// Returns an error if ENA query, filtering, or snapshot write fails.
pub fn select_snapshot(cwd: &Path, args: &EnaSelectArgs) -> Result<()> {
    let species = normalize_species(cwd, &args.species)?;
    let out_path = args.out.clone().map_or_else(
        || {
            default_corpus_root(cwd, &species.species_id, &args.project, &args.corpus_id)
                .join("ENA_METADATA.snapshot.json")
        },
        |p| resolve_path(cwd, &p),
    );
    let query = EnaQuery {
        projects: vec![args.project.clone()],
        samples: Vec::new(),
        extra_accessions: Vec::new(),
        result: EnaResultKind::ReadRun,
    };
    let client = EnaClient::new("bijux-dna/ena-select").context("create ena client")?;
    let records = client.fetch_records(&query).context("fetch ENA records")?;

    let mut selected = Vec::new();
    let mut rejected = Vec::new();
    let mut se_count = 0usize;
    let mut pe_count = 0usize;

    for record in records {
        let row_base = snapshot_row_from_record(&record);
        match validate_record(&record) {
            Ok(layout) => {
                if layout == LayoutKind::Se && se_count >= args.target_se {
                    rejected.push(SelectionRow {
                        reason: "rejected: se target already satisfied".to_string(),
                        ..row_base
                    });
                    continue;
                }
                if layout == LayoutKind::Pe && pe_count >= args.target_pe {
                    rejected.push(SelectionRow {
                        reason: "rejected: pe target already satisfied".to_string(),
                        ..row_base
                    });
                    continue;
                }
                if layout == LayoutKind::Se {
                    se_count += 1;
                } else {
                    pe_count += 1;
                }
                selected.push(SelectionRow {
                    reason: "accepted: metadata + scope checks passed".to_string(),
                    ..row_base
                });
            }
            Err(reason) => rejected.push(SelectionRow {
                reason: format!("rejected: {reason}"),
                ..row_base
            }),
        }
        if se_count >= args.target_se && pe_count >= args.target_pe {
            break;
        }
    }

    if se_count < args.target_se || pe_count < args.target_pe {
        return Err(anyhow!(
            "insufficient accepted records: wanted {} SE + {} PE, got {} SE + {} PE",
            args.target_se,
            args.target_pe,
            se_count,
            pe_count
        ));
    }

    let snapshot = MetadataSnapshot {
        schema_version: "bijux.ena_metadata_snapshot.v3".to_string(),
        species_id: species.species_id,
        species_display: species.species_display,
        project: args.project.clone(),
        corpus_id: args.corpus_id.clone(),
        target_se: args.target_se,
        target_pe: args.target_pe,
        selected,
        rejected,
    };
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&out_path, &snapshot)
        .with_context(|| format!("write {}", out_path.display()))?;
    println!("snapshot={}", out_path.display());
    Ok(())
}

/// # Errors
/// Returns an error if snapshot cannot be loaded, downloads fail, or manifest write fails.
pub fn fetch_from_snapshot(cwd: &Path, args: &EnaFetchArgs) -> Result<()> {
    let species = normalize_species(cwd, &args.species)?;
    let snapshot_path = resolve_path(cwd, &args.snapshot);
    let raw = fs::read_to_string(&snapshot_path)
        .with_context(|| format!("read {}", snapshot_path.display()))?;
    let snapshot: MetadataSnapshot =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", snapshot_path.display()))?;
    if snapshot.species_id != species.species_id {
        return Err(anyhow!(
            "--species mismatch: snapshot has `{}`, input resolved to `{}`",
            snapshot.species_id,
            species.species_id
        ));
    }
    let out_dir = args.out.clone().map_or_else(
        || {
            default_corpus_root(
                cwd,
                &snapshot.species_id,
                &snapshot.project,
                &snapshot.corpus_id,
            )
            .join("raw")
        },
        |p| resolve_path(cwd, &p),
    );
    if snapshot.selected.is_empty() {
        return Err(anyhow!("snapshot has zero selected runs"));
    }
    let records = snapshot
        .selected
        .iter()
        .map(record_from_snapshot_row)
        .collect::<Vec<_>>();

    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    let manifest = EnaRunManifest {
        query: EnaQuery {
            projects: vec![snapshot.project.clone()],
            samples: Vec::new(),
            extra_accessions: Vec::new(),
            result: EnaResultKind::ReadRun,
        },
        source: EnaFileSource::FastqFtp,
        preference: EnaSourcePreference::Https,
        records: records.clone(),
    };
    let dl_cfg = DownloadConfig {
        output_dir: out_dir.clone(),
        jobs: 4,
        retries: 2,
        source: EnaFileSource::FastqFtp,
        preference: EnaSourcePreference::Https,
        dry_run: false,
    };
    let tasks = build_download_tasks(&manifest.records, &dl_cfg);
    let report = download_tasks(&tasks, &dl_cfg).context("download selected ENA FASTQ files")?;
    if report.failed > 0 {
        return Err(anyhow!(
            "ENA download failures: {} files failed",
            report.failed
        ));
    }

    let corpus_root = out_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("--out must point to a corpus raw directory"))?;
    write_manifest(&corpus_root, &snapshot)?;
    set_raw_readonly(&out_dir)?;
    println!("raw_dir={}", out_dir.display());
    println!("downloaded={}", report.downloaded);
    println!("manifest={}", corpus_root.join("MANIFEST.json").display());
    Ok(())
}

fn snapshot_row_from_record(record: &EnaRecord) -> SelectionRow {
    SelectionRow {
        accession: record.accession_label(),
        sample_accession: record.sample_accession.clone(),
        read_layout: record
            .library_layout
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        library_type: record
            .library_strategy
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        instrument: record
            .instrument_model
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        base_count: record.base_count.unwrap_or(0),
        read_count: record.read_count.unwrap_or(0),
        fastq_ftp: record.fastq_ftp.clone(),
        fastq_bytes: record.fastq_bytes.clone(),
        reason: String::new(),
    }
}

fn record_from_snapshot_row(row: &SelectionRow) -> EnaRecord {
    EnaRecord {
        study_accession: None,
        sample_accession: row.sample_accession.clone(),
        experiment_accession: None,
        run_accession: Some(row.accession.clone()),
        analysis_accession: None,
        tax_id: None,
        scientific_name: None,
        library_layout: Some(row.read_layout.clone()),
        library_source: None,
        library_strategy: Some(row.library_type.clone()),
        instrument_model: Some(row.instrument.clone()),
        base_count: Some(row.base_count),
        read_count: Some(row.read_count),
        fastq_bytes: row.fastq_bytes.clone(),
        fastq_ftp: row.fastq_ftp.clone(),
        submitted_ftp: Vec::new(),
        sra_ftp: Vec::new(),
        bam_ftp: Vec::new(),
    }
}

fn validate_record(record: &EnaRecord) -> Result<LayoutKind, String> {
    let layout = detect_layout(record).ok_or_else(|| "unrecognized layout".to_string())?;
    if record.fastq_ftp.is_empty() {
        return Err("missing fastq_ftp".to_string());
    }
    if record.base_count.unwrap_or(0) == 0 || record.read_count.unwrap_or(0) == 0 {
        return Err("missing base_count/read_count".to_string());
    }
    if record
        .instrument_model
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err("missing instrument_model".to_string());
    }
    if record
        .library_strategy
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err("missing library_strategy".to_string());
    }
    if !record.fastq_bytes.is_empty()
        && record
            .fastq_bytes
            .iter()
            .any(|value| *value < MIN_FASTQ_BYTES || *value > MAX_FASTQ_BYTES)
    {
        return Err("fastq_bytes outside expected scope".to_string());
    }
    match layout {
        LayoutKind::Se if record.fastq_ftp.len() != 1 => {
            return Err("SE layout must have exactly one FASTQ".to_string());
        }
        LayoutKind::Pe if record.fastq_ftp.len() != 2 => {
            return Err("PE layout must have exactly two FASTQ".to_string());
        }
        _ => {}
    }
    Ok(layout)
}

fn detect_layout(record: &EnaRecord) -> Option<LayoutKind> {
    if let Some(raw) = record.library_layout.as_deref() {
        let normalized = raw.trim().to_ascii_uppercase();
        if normalized == "SINGLE" {
            return Some(LayoutKind::Se);
        }
        if normalized == "PAIRED" {
            return Some(LayoutKind::Pe);
        }
    }
    match record.fastq_ftp.len() {
        1 => Some(LayoutKind::Se),
        2 => Some(LayoutKind::Pe),
        _ => None,
    }
}

fn write_manifest(corpus_root: &Path, snapshot: &MetadataSnapshot) -> Result<()> {
    let mut files = BTreeMap::new();
    let mut stack = vec![corpus_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let is_fastq = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".fastq.gz"));
            if !is_fastq {
                continue;
            }
            let rel = path.strip_prefix(corpus_root).map_or_else(
                |_| path.display().to_string(),
                |v| v.to_string_lossy().to_string(),
            );
            let digest = bijux_dna_infra::hash_file_sha256(&path)
                .with_context(|| format!("hash {}", path.display()))?;
            files.insert(rel, digest);
        }
    }
    let manifest = CorpusManifest {
        schema_version: "bijux.corpus_manifest.v1",
        species_id: snapshot.species_id.clone(),
        species_display: snapshot.species_display.clone(),
        project: snapshot.project.clone(),
        corpus_id: snapshot.corpus_id.clone(),
        files,
    };
    let path = corpus_root.join("MANIFEST.json");
    bijux_dna_infra::atomic_write_json(&path, &manifest)
        .with_context(|| format!("write {}", path.display()))
}

fn set_raw_readonly(raw_dir: &Path) -> Result<()> {
    let mut stack = vec![raw_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let mut perms = fs::metadata(&path)
                .with_context(|| format!("stat {}", path.display()))?
                .permissions();
            perms.set_readonly(true);
            fs::set_permissions(&path, perms)
                .with_context(|| format!("chmod readonly {}", path.display()))?;
        }
    }
    Ok(())
}

fn resolve_path(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn default_corpus_root(cwd: &Path, species_id: &str, project: &str, corpus_id: &str) -> PathBuf {
    cwd.join("examples").join("bijux-dna-data")
        .join(species_id)
        .join(project)
        .join(corpus_id)
}

fn normalize_species(cwd: &Path, raw: &str) -> Result<SpeciesIdentity> {
    let aliases = load_species_aliases(cwd)?;
    let input = raw.trim();
    if input.is_empty() {
        return Err(anyhow!("species must not be empty"));
    }
    let lowered = input.to_ascii_lowercase();
    let resolved = aliases
        .get(&lowered)
        .cloned()
        .unwrap_or_else(|| input.to_string());
    let tokens = resolved
        .split_whitespace()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();
    if tokens.len() != 2
        || tokens
            .iter()
            .any(|t| !t.chars().all(|c| c.is_ascii_alphabetic()))
    {
        return Err(anyhow!(
            "ambiguous species `{raw}`: provide mapped alias or latin binomial `Genus species`"
        ));
    }
    let genus = tokens[0].to_ascii_lowercase();
    let epithet = tokens[1].to_ascii_lowercase();
    Ok(SpeciesIdentity {
        species_id: format!("{genus}_{epithet}"),
        species_display: format!("{} {}", titlecase(&genus), epithet),
    })
}

fn load_species_aliases(cwd: &Path) -> Result<BTreeMap<String, String>> {
    let path = cwd.join("configs").join("species_aliases.toml");
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let toml_value: toml::Value =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    let mut out = BTreeMap::new();
    let Some(table) = toml_value.get("aliases").and_then(toml::Value::as_table) else {
        return Err(anyhow!("{} missing [aliases] table", path.display()));
    };
    for (k, v) in table {
        let Some(species) = v.as_str() else {
            continue;
        };
        out.insert(k.to_ascii_lowercase(), species.to_string());
    }
    Ok(out)
}

fn titlecase(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = String::new();
    out.push(first.to_ascii_uppercase());
    out.push_str(chars.as_str().to_ascii_lowercase().as_str());
    out
}
