use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct CorpusManifest {
    #[serde(default = "default_manifest_schema")]
    schema_version: String,
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

fn list_inputs(cwd: &Path, corpus: Option<&str>) -> Result<Vec<CorpusInputs>> {
    let mut corpora = Vec::new();
    if let Some(value) = corpus {
        let root = resolve_corpus_root(cwd, value);
        corpora.push(corpus_inputs_for_root(cwd, &root)?);
        return Ok(corpora);
    }

    let data_root = cwd.join("bijux-dna-data");
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
        .unwrap_or("unknown")
        .to_string();
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
