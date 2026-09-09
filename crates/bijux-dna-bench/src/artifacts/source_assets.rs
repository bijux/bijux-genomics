//! Owner: bijux-dna-bench
//! Read-only inspection of benchmark source files, directories, and index-prefix bundles.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, ensure, Context, Result};

/// Storage representation of a benchmark input, independent of execution backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkSourceKind {
    /// One declared input file.
    File,
    /// A directory tree, hashed by sorted relative member paths.
    Directory,
    /// Files sharing a declared index prefix followed by a dot.
    PrefixBundle,
}

impl BenchmarkSourceKind {
    /// Stable spelling used in persisted source-asset manifests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
            Self::PrefixBundle => "prefix_bundle",
        }
    }
}

/// Content identity of a source input before staging or execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkSourceAsset {
    /// Physical representation of the input.
    pub kind: BenchmarkSourceKind,
    /// Caller-declared path, retained for provenance and staging.
    pub source_path: String,
    /// File SHA-256 or the stable sorted-member digest for compound inputs.
    pub checksum_sha256: String,
    /// Total bytes across regular-file members.
    pub size_bytes: u64,
    /// Number of regular-file members represented by the digest.
    pub member_count: usize,
}

/// Inspect a source relative to the caller's root without creating outputs.
///
/// Missing files may denote index-prefix bundles; an absent bundle returns `None`.
/// Existing directory symlinks retain their logical relative paths in the digest.
///
/// # Errors
/// Returns filesystem/hash errors, including cyclic directory traversal.
pub fn inspect_benchmark_source_asset(
    repo_root: &Path,
    source_path: &str,
) -> Result<Option<BenchmarkSourceAsset>> {
    let absolute_source = repo_root.join(source_path);
    if absolute_source.is_file() {
        let checksum_sha256 = bijux_dna_infra::hash_file_sha256(&absolute_source)
            .map_err(|err| anyhow!(err.to_string()))?;
        let size_bytes = absolute_source
            .metadata()
            .with_context(|| format!("read metadata for {}", absolute_source.display()))?
            .len();
        return Ok(Some(BenchmarkSourceAsset {
            kind: BenchmarkSourceKind::File,
            source_path: source_path.to_string(),
            checksum_sha256,
            size_bytes,
            member_count: 1,
        }));
    }
    if absolute_source.is_dir() {
        let (checksum_sha256, size_bytes, member_count) =
            digest_directory_members(&absolute_source, &absolute_source)?;
        return Ok(Some(BenchmarkSourceAsset {
            kind: BenchmarkSourceKind::Directory,
            source_path: source_path.to_string(),
            checksum_sha256,
            size_bytes,
            member_count,
        }));
    }

    resolve_prefix_bundle(&absolute_source, source_path)
}

fn resolve_prefix_bundle(
    absolute_source: &Path,
    source_path: &str,
) -> Result<Option<BenchmarkSourceAsset>> {
    let Some(parent) = absolute_source.parent() else {
        return Ok(None);
    };
    if !parent.exists() {
        return Ok(None);
    }
    let Some(prefix) = absolute_source.file_name().and_then(|value| value.to_str()) else {
        return Ok(None);
    };

    let mut members = fs::read_dir(parent)
        .with_context(|| format!("read {}", parent.display()))?
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with(&format!("{prefix}.")))
        })
        .collect::<Vec<_>>();
    members.sort();

    if members.is_empty() {
        return Ok(None);
    }

    let (checksum_sha256, size_bytes, member_count) = digest_explicit_members(&members, parent)?;
    Ok(Some(BenchmarkSourceAsset {
        kind: BenchmarkSourceKind::PrefixBundle,
        source_path: source_path.to_string(),
        checksum_sha256,
        size_bytes,
        member_count,
    }))
}

fn digest_directory_members(root: &Path, path: &Path) -> Result<(String, u64, usize)> {
    let mut members = Vec::new();
    collect_directory_files(path, &mut members, &mut BTreeSet::new())?;
    members.sort();
    digest_explicit_members(&members, root)
}

fn collect_directory_files(
    path: &Path,
    members: &mut Vec<PathBuf>,
    ancestors: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    let canonical = path.canonicalize().with_context(|| format!("resolve {}", path.display()))?;
    ensure!(ancestors.insert(canonical.clone()), "cyclic source directory at {}", path.display());
    for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
        let entry = entry.with_context(|| format!("read entry in {}", path.display()))?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_directory_files(&entry_path, members, ancestors)?;
        } else if entry_path.is_file() {
            members.push(entry_path);
        }
    }
    ancestors.remove(&canonical);
    Ok(())
}

fn digest_explicit_members(
    members: &[PathBuf],
    relative_root: &Path,
) -> Result<(String, u64, usize)> {
    let mut identity = Vec::new();
    let mut size_bytes = 0u64;

    for member in members {
        let digest =
            bijux_dna_infra::hash_file_sha256(member).map_err(|err| anyhow!(err.to_string()))?;
        let metadata =
            member.metadata().with_context(|| format!("read metadata for {}", member.display()))?;
        let relative_member = member
            .strip_prefix(relative_root)
            .unwrap_or(member)
            .to_string_lossy()
            .replace('\\', "/");
        size_bytes += metadata.len();
        identity.extend_from_slice(relative_member.as_bytes());
        identity.push(0);
        identity.extend_from_slice(digest.as_bytes());
        identity.push(0);
        identity.extend_from_slice(metadata.len().to_string().as_bytes());
        identity.push(b'\n');
    }

    Ok((bijux_dna_infra::hash_bytes_sha256(&identity), size_bytes, members.len()))
}
