use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    files: BTreeMap<String, String>,
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
    }

    let mut sample_to_reads = BTreeMap::<String, (Option<PathBuf>, Option<PathBuf>)>::new();
    for entry in
        fs::read_dir(&normalized).with_context(|| format!("read {}", normalized.display()))?
    {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !name.ends_with(".fastq.gz") || !name.starts_with("sample_") {
            continue;
        }
        let sample = name
            .trim_start_matches("sample_")
            .trim_end_matches(".fastq.gz")
            .replace("_R1", "")
            .replace("_R2", "");
        let slot = sample_to_reads.entry(sample).or_default();
        if name.ends_with("_R1.fastq.gz") {
            slot.0 = Some(path);
        } else if name.ends_with("_R2.fastq.gz") {
            slot.1 = Some(path);
        }
    }
    if sample_to_reads.is_empty() {
        return Err(anyhow!("no normalized sample_*.fastq.gz files found"));
    }

    for (sample, (r1, r2)) in sample_to_reads {
        let Some(r1_path) = r1 else {
            return Err(anyhow!("sample {sample} missing R1"));
        };
        let h1 = first_fastq_header(&r1_path)?;
        if h1.trim().is_empty() {
            return Err(anyhow!("sample {sample} has empty R1 header"));
        }
        if let Some(r2_path) = r2 {
            let h2 = first_fastq_header(&r2_path)?;
            let n1 = normalize_read_header(&h1);
            let n2 = normalize_read_header(&h2);
            if n1 != n2 {
                return Err(anyhow!(
                    "sample {sample} paired read-name mismatch: `{n1}` vs `{n2}`"
                ));
            }
        }
    }
    println!("corpus validation ok: {}", root.display());
    Ok(())
}

/// # Errors
/// Returns an error if corpus enumeration fails.
pub fn list_corpus_json(cwd: &Path) -> Result<()> {
    let data_root = cwd.join("bijux-dna-data");
    let mut corpora = Vec::new();
    if data_root.exists() {
        for entry in
            fs::read_dir(&data_root).with_context(|| format!("read {}", data_root.display()))?
        {
            let path = entry?.path();
            if !path.is_dir() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("unknown")
                .to_string();
            let normalized = path.join("normalized");
            if !normalized.exists() {
                continue;
            }
            let mut files = fs::read_dir(&normalized)
                .with_context(|| format!("read {}", normalized.display()))?
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .filter_map(|path| {
                    path.strip_prefix(cwd)
                        .ok()
                        .map(|rel| rel.to_string_lossy().to_string())
                })
                .collect::<Vec<_>>();
            files.sort();
            files.dedup();
            corpora.push(CorpusInputs {
                corpus: name,
                files,
            });
        }
    }
    corpora.sort_by(|a, b| a.corpus.cmp(&b.corpus));
    let payload = CorpusListJson {
        schema_version: "bijux.corpus.list.v1",
        corpora,
    };
    crate::commands::cli::render::json::print_pretty(&payload)
}

fn resolve_corpus_root(cwd: &Path, corpus: &str) -> PathBuf {
    let raw = PathBuf::from(corpus);
    if raw.is_absolute() {
        return raw;
    }
    if corpus.contains('/') {
        return cwd.join(corpus);
    }
    cwd.join("bijux-dna-data").join(corpus)
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

fn normalize_read_header(header: &str) -> String {
    let raw = header.trim_start_matches('@');
    raw.trim_end_matches("/1")
        .trim_end_matches("/2")
        .split_whitespace()
        .next()
        .unwrap_or(raw)
        .to_string()
}
