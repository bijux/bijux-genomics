use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use sha2::Digest as _;

use super::path_display;

pub(super) fn artifact_bundle_manifest_fields(
    path_key: &str,
    digest_prefix: &str,
    path: &Path,
) -> Result<std::collections::BTreeMap<String, serde_json::Value>> {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(path_key.to_string(), serde_json::Value::String(path.display().to_string()));
    fields.insert(
        format!("{digest_prefix}_digest"),
        serde_json::Value::String(sha256_artifact_bundle(path)?),
    );
    fields.insert(
        format!("{digest_prefix}_size_bytes"),
        serde_json::Value::Number(serde_json::Number::from(artifact_bundle_size_bytes(path)?)),
    );
    if let Some(lineage_json) = resolve_artifact_lineage_json(path) {
        fields.insert(
            format!("{digest_prefix}_lineage_json"),
            serde_json::Value::String(lineage_json.display().to_string()),
        );
        fields.insert(
            format!("{digest_prefix}_lineage_digest"),
            serde_json::Value::String(sha256_file_hex(&lineage_json)?),
        );
    }
    Ok(fields)
}

fn artifact_bundle_members(path: &Path) -> Result<Vec<PathBuf>> {
    if path.exists() {
        return Ok(vec![path.to_path_buf()]);
    }
    let Some(parent) = path.parent() else {
        return Ok(Vec::new());
    };
    if !parent.is_dir() {
        return Ok(Vec::new());
    }
    let Some(prefix) = path.file_name().and_then(|row| row.to_str()) else {
        return Ok(Vec::new());
    };
    let mut members = fs::read_dir(parent)
        .with_context(|| format!("read {}", parent.display()))?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_str()?;
            (name.starts_with(prefix) && (path.is_file() || path.is_dir())).then_some(path)
        })
        .collect::<Vec<_>>();
    members.sort();
    Ok(members)
}

pub(crate) fn artifact_bundle_size_bytes(path: &Path) -> Result<u64> {
    let mut total = 0_u64;
    for member in artifact_bundle_members(path)? {
        if member.is_file() {
            total += member.metadata().with_context(|| format!("stat {}", member.display()))?.len();
            continue;
        }
        let mut nested = member
            .read_dir()
            .with_context(|| format!("read {}", member.display()))?
            .filter_map(|entry| entry.ok().map(|row| row.path()))
            .collect::<Vec<_>>();
        while let Some(candidate) = nested.pop() {
            if candidate.is_dir() {
                let children = candidate
                    .read_dir()
                    .with_context(|| format!("read {}", candidate.display()))?
                    .filter_map(|entry| entry.ok().map(|row| row.path()))
                    .collect::<Vec<_>>();
                nested.extend(children);
                continue;
            }
            total += candidate
                .metadata()
                .with_context(|| format!("stat {}", candidate.display()))?
                .len();
        }
    }
    Ok(total)
}

pub(crate) fn sha256_artifact_bundle(path: &Path) -> Result<String> {
    let members = artifact_bundle_members(path)?;
    if members.is_empty() {
        return Err(anyhow!("missing artifact bundle: {}", path.display()));
    }
    let mut digest = sha2::Sha256::new();
    for member in members {
        if member.is_file() {
            let name = member
                .file_name()
                .and_then(|row| row.to_str())
                .ok_or_else(|| anyhow!("invalid artifact bundle member {}", member.display()))?;
            digest.update(name.as_bytes());
            digest.update(b"\0file\0");
            digest.update(sha256_file_hex(&member)?.as_bytes());
            continue;
        }
        let parent = member
            .parent()
            .ok_or_else(|| anyhow!("artifact bundle member missing parent {}", member.display()))?;
        let mut nested = collect_sorted_paths(&member)?;
        for path in nested.drain(..) {
            if path == member {
                continue;
            }
            let relative = path.strip_prefix(parent).with_context(|| {
                format!("strip prefix {} from {}", parent.display(), path.display())
            })?;
            digest.update(path_display(relative).as_bytes());
            if path.is_dir() {
                digest.update(b"\0dir\0");
                continue;
            }
            digest.update(b"\0file\0");
            digest.update(sha256_file_hex(&path)?.as_bytes());
        }
    }
    Ok(sha256_hex(&digest.finalize()))
}

fn collect_sorted_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut all = vec![root.to_path_buf()];
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        if !path.is_dir() {
            continue;
        }
        let mut children = fs::read_dir(&path)
            .with_context(|| format!("read {}", path.display()))?
            .filter_map(|entry| entry.ok().map(|row| row.path()))
            .collect::<Vec<_>>();
        children.sort();
        for child in &children {
            all.push(child.clone());
        }
        children.reverse();
        stack.extend(children);
    }
    all.sort();
    Ok(all)
}

pub(crate) fn resolve_artifact_lineage_json(path: &Path) -> Option<PathBuf> {
    let resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let candidate = if resolved.is_dir() {
        resolved.join("lineage.json")
    } else {
        resolved.parent()?.join("lineage.json")
    };
    candidate.is_file().then_some(candidate)
}

pub(crate) fn sha256_file_hex(path: &Path) -> Result<String> {
    let mut handle = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut digest = sha2::Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        use std::io::Read as _;
        let read = handle.read(&mut buffer).with_context(|| format!("read {}", path.display()))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(sha256_hex(&digest.finalize()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}
