use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub(crate) fn check_empty_modules(files: &[PathBuf]) -> Result<()> {
    for path in files {
        if path.file_name().and_then(|name| name.to_str()) != Some("mod.rs") {
            continue;
        }
        let content = fs::read_to_string(path)?;
        let mut meaningful = 0usize;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
                continue;
            }
            if is_module_declaration(trimmed) {
                continue;
            }
            meaningful += 1;
        }
        if meaningful == 0 {
            anyhow::bail!("empty module file (only mod re-exports): {}", path.display());
        }
    }
    Ok(())
}

pub(crate) fn check_mod_reexports_only(files: &[PathBuf]) -> Result<()> {
    for path in files {
        if path.file_name().and_then(|name| name.to_str()) != Some("mod.rs") {
            continue;
        }
        if !path.to_string_lossy().ends_with("stages/mod.rs") {
            continue;
        }
        let content = fs::read_to_string(path)?;
        let mut meaningful = 0usize;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            if trimmed.starts_with("#[") {
                continue;
            }
            if is_module_declaration(trimmed) {
                continue;
            }
            meaningful += 1;
        }
        if meaningful == 0 {
            anyhow::bail!("stages mod.rs contains only re-exports: {}", path.display());
        }
    }
    Ok(())
}

fn is_module_declaration(line: &str) -> bool {
    line.starts_with("mod ") || line.starts_with("pub mod ") || line.starts_with("pub(")
}
