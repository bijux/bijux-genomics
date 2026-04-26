use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::checks::path_match::path_has_allowed_suffix;
use crate::GuardrailConfig;

pub(crate) fn check_mod_only_dirs(src_dir: &Path, config: &GuardrailConfig) -> Result<()> {
    for entry in WalkDir::new(src_dir).min_depth(1).max_depth(10) {
        let entry = entry?;
        if !entry.file_type().is_dir() {
            continue;
        }
        if entry.path() == src_dir {
            continue;
        }
        if entry.path().components().any(|component| component.as_os_str() == "tests") {
            continue;
        }
        if path_has_allowed_suffix(entry.path(), &config.allow_mod_only_dirs) {
            continue;
        }
        let mut rs_files = Vec::new();
        for child in fs::read_dir(entry.path())? {
            let child = child?;
            let path = child.path();
            if path.is_file() && path.extension().and_then(|suffix| suffix.to_str()) == Some("rs") {
                if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                    rs_files.push(name.to_string());
                }
            }
        }
        if rs_files.is_empty() {
            continue;
        }
        let allowed = rs_files.iter().all(|name| name == "mod.rs" || name == "tests.rs");
        if allowed && rs_files.iter().any(|name| name == "mod.rs") && rs_files.len() <= 2 {
            anyhow::bail!(
                "module directory contains only mod.rs (and optionally tests.rs): {}",
                entry.path().display()
            );
        }
    }
    Ok(())
}
