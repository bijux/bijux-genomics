//! Owner: bijux-dna-bench
//! Bounded legacy manifest decoding and current content identity checks.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

const MAX_MANIFEST_BYTES: u64 = 64 * 1024 * 1024;
const SCHEMA: &str = "bijux.corpus_manifest.v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CorpusManifest {
    // Early corpus producers omitted this field; their representation is exactly v1.
    #[serde(default = "legacy_schema")]
    schema_version: String,
    #[serde(deserialize_with = "unique_files")]
    pub(super) files: BTreeMap<String, String>,
}

fn legacy_schema() -> String {
    SCHEMA.to_string()
}

fn unique_files<'de, D: Deserializer<'de>>(
    decoder: D,
) -> Result<BTreeMap<String, String>, D::Error> {
    struct Files;
    impl<'de> Visitor<'de> for Files {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map of unique corpus paths to SHA-256 digests")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut files = BTreeMap::new();
            while let Some((path, digest)) = map.next_entry::<String, String>()? {
                if files.insert(path.clone(), digest).is_some() {
                    return Err(serde::de::Error::custom(format!("repeated manifest path {path}")));
                }
            }
            Ok(files)
        }
    }
    decoder.deserialize_map(Files)
}

pub(super) fn load(root: &Path) -> Result<CorpusManifest> {
    let path = contained_regular_file(root, "MANIFEST.json")?;
    let file = File::open(&path).with_context(|| format!("open {}", path.display()))?;
    if file.metadata()?.len() > MAX_MANIFEST_BYTES {
        return Err(anyhow!("corpus manifest exceeds {MAX_MANIFEST_BYTES} bytes"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_MANIFEST_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(anyhow!("corpus manifest exceeds {MAX_MANIFEST_BYTES} bytes"));
    }
    let manifest: CorpusManifest =
        serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?;
    if manifest.schema_version != SCHEMA || manifest.files.is_empty() {
        return Err(anyhow!("unsupported corpus manifest schema or empty files map"));
    }
    for (relative, digest) in &manifest.files {
        validate_relative_path(relative)?;
        if digest.len() != 64
            || !digest.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(anyhow!("invalid SHA-256 digest for {relative}"));
        }
    }
    Ok(manifest)
}

fn validate_relative_path(relative: &str) -> Result<()> {
    if relative.contains('\\')
        || relative.split('/').any(|part| matches!(part, "" | "." | ".."))
        || Path::new(relative).components().any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(anyhow!("manifest path must be canonical and relative: {relative}"));
    }
    Ok(())
}

fn contained_regular_file(root: &Path, relative: &str) -> Result<PathBuf> {
    validate_relative_path(relative)?;
    let root = root.canonicalize().context("resolve corpus root")?;
    let mut path = root.clone();
    for part in Path::new(relative).components() {
        path.push(part);
        if fs::symlink_metadata(&path)
            .with_context(|| format!("inspect {}", path.display()))?
            .file_type()
            .is_symlink()
        {
            return Err(anyhow!("corpus authority cannot follow a symlink: {}", path.display()));
        }
    }
    let resolved = path.canonicalize()?;
    if !resolved.starts_with(&root) || !fs::metadata(&resolved)?.is_file() {
        return Err(anyhow!(
            "corpus authority must be a contained regular file: {}",
            path.display()
        ));
    }
    Ok(resolved)
}

pub(super) fn verify_content(root: &Path, manifest: &CorpusManifest) -> Result<()> {
    // Hash reads in a fixed-size buffer; a large HPC corpus must not be loaded into memory.
    for (relative, expected) in &manifest.files {
        let path = contained_regular_file(root, relative)?;
        let actual = bijux_dna_infra::hash_file_sha256(&path)?;
        if &actual != expected {
            return Err(anyhow!(
                "corpus checksum mismatch for {relative}: expected {expected}, found {actual}"
            ));
        }
    }
    Ok(())
}
