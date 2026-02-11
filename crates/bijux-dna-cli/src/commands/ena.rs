use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_db_ena::client::EnaClient;
use bijux_dna_db_ena::download::{build_download_tasks, download_tasks, DownloadConfig};
use bijux_dna_db_ena::model::{
    EnaFileSource, EnaQuery, EnaRecord, EnaResultKind, EnaRunManifest, EnaSourcePreference,
};
use serde::Serialize;

use crate::commands::cli::EnaFetchArgs;

const MIN_FASTQ_BYTES: u64 = 1_000_000;
const MAX_FASTQ_BYTES: u64 = 200_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutKind {
    Se,
    Pe,
}

#[derive(Debug, Serialize)]
struct MetadataSnapshot {
    schema_version: &'static str,
    project: String,
    selected: Vec<MetadataRow>,
    rejected: Vec<RejectedRow>,
}

#[derive(Debug, Serialize)]
struct MetadataRow {
    run_accession: String,
    sample_accession: Option<String>,
    read_layout: String,
    library_type: String,
    instrument: String,
    base_count: u64,
    read_count: u64,
    fastq_ftp: Vec<String>,
    fastq_bytes: Vec<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct RejectedRow {
    accession: String,
    reason: String,
}

#[derive(Debug, Serialize)]
struct CorpusManifest {
    schema_version: &'static str,
    files: BTreeMap<String, String>,
}

/// # Errors
/// Returns an error if ENA fetch, filtering, download, or corpus materialization fails.
pub fn fetch_corpus(cwd: &Path, args: &EnaFetchArgs) -> Result<()> {
    let limits = parse_limits(&args.limits)?;
    let out_dir = resolve_path(cwd, &args.out);
    let raw_dir = out_dir.clone();
    let corpus_root = raw_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("--out must point to a corpus raw directory"))?;
    let normalized_dir = corpus_root.join("normalized");
    if normalized_dir.starts_with(&raw_dir) || raw_dir.starts_with(&normalized_dir) {
        return Err(anyhow!(
            "invalid corpus layout: raw and normalized must be separate directories"
        ));
    }
    fs::create_dir_all(&raw_dir).with_context(|| format!("create {}", raw_dir.display()))?;
    fs::create_dir_all(&normalized_dir)
        .with_context(|| format!("create {}", normalized_dir.display()))?;

    let query = EnaQuery {
        projects: vec![args.project.clone()],
        samples: Vec::new(),
        extra_accessions: Vec::new(),
        result: EnaResultKind::ReadRun,
    };
    let client = EnaClient::new("bijux-dna/ena-fetch").context("create ena client")?;
    let records = client.fetch_records(&query).context("fetch ENA records")?;

    let mut selected = Vec::new();
    let mut rejected = Vec::new();
    let mut se_count = 0_usize;
    let mut pe_count = 0_usize;
    for record in records {
        let accession = record.accession_label();
        match validate_record(&record) {
            Ok(layout) => {
                if layout == LayoutKind::Se && se_count >= limits.se {
                    continue;
                }
                if layout == LayoutKind::Pe && pe_count >= limits.pe {
                    continue;
                }
                if layout == LayoutKind::Se {
                    se_count += 1;
                } else {
                    pe_count += 1;
                }
                selected.push(record);
            }
            Err(reason) => rejected.push(RejectedRow { accession, reason }),
        }
        if se_count >= limits.se && pe_count >= limits.pe {
            break;
        }
    }
    if se_count < limits.se || pe_count < limits.pe {
        return Err(anyhow!(
            "insufficient accepted records: wanted {} SE + {} PE, got {} SE + {} PE",
            limits.se,
            limits.pe,
            se_count,
            pe_count
        ));
    }

    let manifest = EnaRunManifest {
        query,
        source: EnaFileSource::FastqFtp,
        preference: EnaSourcePreference::Https,
        records: selected.clone(),
    };
    let dl_cfg = DownloadConfig {
        output_dir: raw_dir.clone(),
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

    materialize_normalized(&selected, &raw_dir, &normalized_dir)?;
    write_metadata_snapshot(&corpus_root, &args.project, &selected, &rejected)?;
    write_manifest(&corpus_root)?;
    println!("corpus_root={}", corpus_root.display());
    println!("selected_se={se_count}");
    println!("selected_pe={pe_count}");
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct LayoutLimits {
    se: usize,
    pe: usize,
}

fn parse_limits(items: &[String]) -> Result<LayoutLimits> {
    let mut limits = LayoutLimits { se: 0, pe: 0 };
    for item in items {
        let (count_raw, layout_raw) = item
            .split_once('-')
            .ok_or_else(|| anyhow!("invalid --limit `{item}` expected <count>-se|pe"))?;
        let count = count_raw
            .parse::<usize>()
            .with_context(|| format!("invalid --limit count in `{item}`"))?;
        match layout_raw.to_ascii_lowercase().as_str() {
            "se" => limits.se = count,
            "pe" => limits.pe = count,
            _ => return Err(anyhow!("invalid --limit layout in `{item}` expected se|pe")),
        }
    }
    if limits.se == 0 || limits.pe == 0 {
        return Err(anyhow!(
            "--limit must include both se and pe, e.g. --limit 10-se --limit 10-pe"
        ));
    }
    Ok(limits)
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

fn materialize_normalized(
    records: &[EnaRecord],
    raw_dir: &Path,
    normalized_dir: &Path,
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for record in records {
        let Some(layout) = detect_layout(record) else {
            return Err(anyhow!(
                "record {} has unknown layout",
                record.accession_label()
            ));
        };
        let accession = record.accession_label();
        let sample_id = accession.replace('-', "_");
        if !seen.insert(sample_id.clone()) {
            return Err(anyhow!("duplicate normalized sample id: {sample_id}"));
        }
        let src_dir = raw_dir.join(&accession);
        let mut files = fs::read_dir(&src_dir)
            .with_context(|| format!("read {}", src_dir.display()))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("gz"))
            .collect::<Vec<_>>();
        files.sort();
        match layout {
            LayoutKind::Se if files.len() != 1 => {
                return Err(anyhow!(
                    "SE accession {accession} must have exactly one FASTQ"
                ));
            }
            LayoutKind::Pe if files.len() != 2 => {
                return Err(anyhow!(
                    "PE accession {accession} must have exactly two FASTQ"
                ));
            }
            _ => {}
        }
        let r1 = normalized_dir.join(format!("sample_{sample_id}_R1.fastq.gz"));
        fs::copy(&files[0], &r1)
            .with_context(|| format!("copy {} -> {}", files[0].display(), r1.display()))?;
        if layout == LayoutKind::Pe {
            let r2 = normalized_dir.join(format!("sample_{sample_id}_R2.fastq.gz"));
            fs::copy(&files[1], &r2)
                .with_context(|| format!("copy {} -> {}", files[1].display(), r2.display()))?;
        }
    }
    Ok(())
}

fn write_metadata_snapshot(
    corpus_root: &Path,
    project: &str,
    selected: &[EnaRecord],
    rejected: &[RejectedRow],
) -> Result<()> {
    let selected_rows = selected
        .iter()
        .map(|record| MetadataRow {
            run_accession: record.accession_label(),
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
        })
        .collect::<Vec<_>>();
    let snapshot = MetadataSnapshot {
        schema_version: "bijux.ena_metadata_snapshot.v1",
        project: project.to_string(),
        selected: selected_rows,
        rejected: rejected.to_vec(),
    };
    let path = corpus_root.join("ENA_METADATA.snapshot.json");
    bijux_dna_infra::atomic_write_json(&path, &snapshot)
        .with_context(|| format!("write {}", path.display()))
}

fn write_manifest(corpus_root: &Path) -> Result<()> {
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
        files,
    };
    let path = corpus_root.join("MANIFEST.json");
    bijux_dna_infra::atomic_write_json(&path, &manifest)
        .with_context(|| format!("write {}", path.display()))
}

fn resolve_path(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}
