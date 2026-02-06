use std::path::Path;

use crate::foundation::{BijuxError, Result};

use crate::contract::ExecutionContract;

/// # Errors
/// Returns an error if expected outputs are missing or invalid.
pub fn validate_execution_outputs(contract: &ExecutionContract, out_dir: &Path) -> Result<()> {
    let outputs = collect_outputs(out_dir)?;

    for forbidden in &contract.forbidden_outputs {
        if outputs.iter().any(|path| matches_pattern(path, forbidden)) {
            return Err(BijuxError::contract(format!(
                "forbidden output produced: {forbidden}"
            )));
        }
    }

    for expected in &contract.expected_outputs {
        if !outputs.iter().any(|path| matches_pattern(path, expected)) {
            return Err(BijuxError::contract(format!(
                "expected output missing: {expected}"
            )));
        }
    }

    if contract.forbid_unexpected_outputs {
        for output in &outputs {
            if !contract
                .expected_outputs
                .iter()
                .any(|pattern| matches_pattern(output, pattern))
            {
                return Err(BijuxError::contract(format!(
                    "unexpected output produced: {output}"
                )));
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
    for entry in std::fs::read_dir(dir)
        .map_err(|err| BijuxError::Io(format!("read dir {}: {err}", dir.display())))?
    {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
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
