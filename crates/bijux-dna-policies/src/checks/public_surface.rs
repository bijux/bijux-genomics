use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

use crate::GuardrailConfig;

pub(crate) fn check_pub_items(files: &[PathBuf], config: &GuardrailConfig) -> Result<()> {
    let pub_re = Regex::new(
        r"^\s*pub(?:\s*\([^)]*\))?\s+(struct|enum|fn|type|trait|const|static|use|mod)\b",
    )?;
    for path in files {
        let content = fs::read_to_string(path)?;
        let count = content.lines().filter(|line| pub_re.is_match(line)).count();
        if count > config.max_pub_items_per_file {
            anyhow::bail!(
                "{} has {} pub items (max {})",
                path.display(),
                count,
                config.max_pub_items_per_file
            );
        }
    }
    Ok(())
}

pub(crate) fn check_pub_use_spam(files: &[PathBuf], config: &GuardrailConfig) -> Result<()> {
    let pub_use_re = Regex::new(r"^\s*pub(?:\s*\([^)]*\))?\s+use\b")?;
    for path in files {
        let content = fs::read_to_string(path)?;
        let count = content.lines().filter(|line| pub_use_re.is_match(line)).count();
        if count > config.max_pub_use_per_file {
            anyhow::bail!(
                "{} has {} pub use re-exports (max {})",
                path.display(),
                count,
                config.max_pub_use_per_file
            );
        }
    }
    Ok(())
}
