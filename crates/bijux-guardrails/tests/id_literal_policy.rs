use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn is_allowed_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    if path_str.contains("/crates/bijux-stages-") {
        return true;
    }
    if path_str.contains("/crates/bijux-domain-") {
        return true;
    }
    if path_str.contains("/crates/bijux-pipelines/") {
        return true;
    }
    if path_str.contains("/crates/bijux-analyze/") {
        return true;
    }
    if path_str.contains("/crates/bijux-bench/") {
        return true;
    }
    if path_str.contains("/crates/bijux-cli/") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-core/src/metrics_registry.rs") {
        return true;
    }
    if path_str.contains("/tests/") {
        return true;
    }
    false
}

#[test]
fn raw_stage_ids_are_confined_to_registries() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let patterns = ["\"fastq.", "\"bam.", "\"cross.", "\"core."];
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if is_allowed_path(entry.path()) {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        for pattern in patterns {
            if content.contains(pattern) {
                offenders.push(format!(
                    "{} contains raw id literal {pattern}",
                    entry.path().display()
                ));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "raw stage ids must be confined to registries/stage crates:\n{}",
        offenders.join("\n")
    );
}
