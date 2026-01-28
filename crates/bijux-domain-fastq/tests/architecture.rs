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

fn assert_no_imports(dir: &str, forbidden: &[&str]) {
    let mut files = Vec::new();
    collect_rs_files(Path::new(dir), &mut files);
    for file in files {
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        for needle in forbidden {
            assert!(
                !contents.contains(needle),
                "forbidden import in {}: {}",
                file.display(),
                needle
            );
        }
    }
}

#[test]
fn stages_do_not_import_analyze() {
    assert_no_imports(
        "crates/bijux-domain-fastq/src/stages",
        &["crate::analyze::", "super::analyze::"],
    );
}

#[test]
fn analyze_does_not_import_stages() {
    assert_no_imports(
        "crates/bijux-domain-fastq/src/analyze",
        &["crate::stages::", "super::stages::"],
    );
}

#[test]
fn metrics_does_not_import_stages() {
    assert_no_imports(
        "crates/bijux-domain-fastq/src/metrics",
        &["crate::stages::", "super::stages::"],
    );
}

#[test]
fn domain_has_no_engine_or_environment_dependency() {
    assert_no_imports(
        "crates/bijux-domain-fastq/src",
        &["bijux_engine", "bijux_environment"],
    );
}
