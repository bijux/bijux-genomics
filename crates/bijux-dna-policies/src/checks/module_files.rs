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
        let mut declared_modules = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
                continue;
            }
            if is_module_declaration(trimmed) {
                if let Some(name) = declared_module_name(trimmed) {
                    declared_modules.push(name.to_string());
                }
                continue;
            }
            meaningful += 1;
        }
        if meaningful == 0 && declared_modules_have_sources(path, &declared_modules) {
            continue;
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

fn declared_module_name(line: &str) -> Option<&str> {
    let declaration = line
        .strip_prefix("pub(crate) mod ")
        .or_else(|| line.strip_prefix("pub(super) mod "))
        .or_else(|| line.strip_prefix("pub mod "))
        .or_else(|| line.strip_prefix("mod "))?;
    declaration
        .trim()
        .trim_end_matches(';')
        .split_whitespace()
        .next()
}

fn declared_modules_have_sources(path: &std::path::Path, modules: &[String]) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    !modules.is_empty()
        && modules.iter().all(|module| {
            parent.join(format!("{module}.rs")).is_file()
                || parent.join(module).join("mod.rs").is_file()
        })
}
