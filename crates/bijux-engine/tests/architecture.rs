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

fn assert_no_fastq_terms(dir: &str) {
    let mut files = Vec::new();
    collect_rs_files(Path::new(dir), &mut files);
    for file in files {
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        assert!(
            !contents.contains("fastq."),
            "fastq term in engine core: {}",
            file.display()
        );
    }
}

#[test]
fn executor_does_not_import_planner_observer_validator() {
    assert_no_imports(
        "crates/bijux-engine/src/executor",
        &[
            "crate::planner::",
            "crate::observer::",
            "crate::validator::",
        ],
    );
}

#[test]
fn observer_does_not_import_executor() {
    assert_no_imports("crates/bijux-engine/src/observer", &["crate::executor::"]);
}

#[test]
fn validator_does_not_import_executor() {
    assert_no_imports("crates/bijux-engine/src/validator", &["crate::executor::"]);
}

#[test]
fn engine_core_is_fastq_agnostic() {
    assert_no_fastq_terms("crates/bijux-engine/src/planner");
    assert_no_fastq_terms("crates/bijux-engine/src/executor");
    assert_no_fastq_terms("crates/bijux-engine/src/observer");
    assert_no_fastq_terms("crates/bijux-engine/src/validator");
    assert_no_fastq_terms("crates/bijux-engine/src/types");
    assert_no_fastq_terms("crates/bijux-engine/src/errors");
}
