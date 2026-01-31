use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn no_cross_layer_calls() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    for file in files {
        let path_str = file.to_string_lossy();
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        if path_str.contains("/load/") {
            assert!(
                !contents.contains("crate::decision"),
                "load must not depend on decision: {path_str}"
            );
            assert!(
                !contents.contains("crate::report"),
                "load must not depend on report: {path_str}"
            );
        }
        if path_str.contains("/report/") {
            assert!(
                !contents.contains("crate::load"),
                "report must not depend on load: {path_str}"
            );
        }
        if path_str.contains("/decision/") {
            assert!(
                !contents.contains("crate::report"),
                "decision must not depend on report: {path_str}"
            );
            assert!(
                !contents.contains("crate::load"),
                "decision must not depend on load: {path_str}"
            );
        }
    }
}

#[test]
fn public_api_is_small() -> anyhow::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_path = manifest_dir.join("src").join("lib.rs");
    let raw = fs::read_to_string(&lib_path)?;
    let mut offenders = Vec::new();
    let mut skip_next_pub = false;
    let mut in_pub_block = 0usize;
    let allowed = [
        "pub mod aggregate;",
        "pub mod contract;",
        "pub mod decision;",
        "pub mod facts;",
        "pub mod load;",
        "pub mod model;",
        "pub mod report;",
        "pub mod semantic;",
        "pub use contract::{analyze_contract_v1, AnalyzeContractV1};",
        "pub struct AnalyzeInput {",
        "pub enum AnalyzeSources {",
        "pub struct AnalyzeOptions {",
        "pub enum AnalyzeMode {",
        "pub struct RenderOptions {",
        "pub struct AnalyzeOutput {",
        "pub fn analyze_run(input: &AnalyzeInput) -> anyhow::Result<AnalyzeOutput> {",
        "pub use aggregate::*;",
        "pub use failure::*;",
        "pub use load::*;",
        "pub use semantic::*;",
        "pub use report::*;",
        "pub use decision::compare::compare_runs;",
        "pub use bijux_core::metrics::MetricSet;",
        "pub mod failure;",
        "pub use crate::decision::score::{",
        "pub mod compare {",
        "pub mod ranking {",
        "pub use crate::decision::compare::*;",
        "pub use crate::decision::score::*;",
    ];
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[cfg(test)]") {
            skip_next_pub = true;
            continue;
        }
        if in_pub_block > 0 {
            if trimmed.contains('{') {
                in_pub_block += trimmed.matches('{').count();
            }
            if trimmed.contains('}') {
                let closes = trimmed.matches('}').count();
                in_pub_block = in_pub_block.saturating_sub(closes);
            }
            continue;
        }
        if skip_next_pub {
            if trimmed.starts_with("pub ") {
                skip_next_pub = false;
                continue;
            }
            if !trimmed.is_empty() {
                skip_next_pub = false;
            }
        }
        if trimmed.starts_with("pub ") {
            if (trimmed.starts_with("pub struct ") || trimmed.starts_with("pub enum "))
                && trimmed.contains('{')
            {
                in_pub_block = trimmed.matches('{').count();
            }
            if !allowed.iter().any(|allowed| trimmed.starts_with(allowed)) {
                offenders.push(trimmed.to_string());
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "unexpected public items in lib.rs: {offenders:?}"
    );
    Ok(())
}

#[test]
fn no_new_top_level_modules_without_owner() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut modules = Vec::new();
    let Ok(entries) = fs::read_dir(&src_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) == Some("lib.rs") {
            continue;
        }
        if path.is_dir() {
            let mod_rs = path.join("mod.rs");
            if mod_rs.exists() {
                modules.push(mod_rs);
            }
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            modules.push(path);
        }
    }

    let mut offenders = Vec::new();
    for module in modules {
        let Ok(contents) = fs::read_to_string(&module) else {
            continue;
        };
        let mut has_owner = false;
        for line in contents.lines().take(5) {
            if line.trim().starts_with("//!") && line.contains("Owner:") {
                has_owner = true;
                break;
            }
            if !line.trim().is_empty() && !line.trim().starts_with("//!") {
                break;
            }
        }
        if !has_owner {
            offenders.push(module.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "missing module owner doc comments: {offenders:?}"
    );
}
