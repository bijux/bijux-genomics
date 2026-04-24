use std::fmt::Write as _;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::ExecutionContract;
use sha2::Digest;

/// # Errors
/// Returns an error if the file cannot be read.
pub fn hash_file_sha256(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).context("read file for hash")?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(&bytes);
    Ok(sha256_hex(hasher.finalize()))
}

/// # Errors
/// Returns an error when the output directory violates the contract.
pub fn validate_execution_outputs(contract: &ExecutionContract, out_dir: &Path) -> Result<()> {
    let outputs = collect_outputs(out_dir)?;

    for forbidden in &contract.forbidden_outputs {
        if outputs.iter().any(|path| matches_pattern(path, forbidden)) {
            return Err(anyhow!("forbidden output produced: {forbidden}"));
        }
    }

    for expected in &contract.expected_outputs {
        if !outputs.iter().any(|path| matches_pattern(path, expected)) {
            return Err(anyhow!("expected output missing: {expected}"));
        }
    }

    if contract.forbid_unexpected_outputs {
        for output in &outputs {
            if !contract.expected_outputs.iter().any(|pattern| matches_pattern(output, pattern)) {
                return Err(anyhow!("unexpected output produced: {output}"));
            }
        }
    }

    Ok(())
}

fn collect_outputs(root: &Path) -> Result<Vec<String>> {
    let mut results = Vec::new();
    walk_outputs(root, root, &mut results)?;
    Ok(results)
}

fn walk_outputs(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("read dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        if path.is_dir() {
            walk_outputs(root, &path, out)?;
        } else if path.is_file() {
            out.push(rel);
        }
    }
    Ok(())
}

fn matches_pattern(value: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return value == pattern;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0usize;
    let starts_with_wildcard = pattern.starts_with('*');
    let ends_with_wildcard = pattern.ends_with('*');

    for (idx, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if let Some(found) = value[pos..].find(part) {
            if idx == 0 && !starts_with_wildcard && found != 0 {
                return false;
            }
            pos += found + part.len();
        } else {
            return false;
        }
    }

    if !ends_with_wildcard {
        if let Some(last) = parts.last() {
            if !last.is_empty() && !value.ends_with(last) {
                return false;
            }
        }
    }
    true
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
