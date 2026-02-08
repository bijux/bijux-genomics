use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .map(walkdir::DirEntry::into_path)
        .collect()
}

#[test]
fn foundation_does_not_depend_on_contract() {
    let root = crate_root().join("src").join("foundation");
    let mut offenders = Vec::new();
    for path in collect_rs_files(&root) {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read source {}: {err}", path.display()));
        if content.contains("crate::contract") {
            offenders.push(path.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "foundation must not depend on contract:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn contract_does_not_depend_on_foundation_internals() {
    let root = crate_root().join("src").join("contract");
    let mut offenders = Vec::new();
    for path in collect_rs_files(&root) {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read source {}: {err}", path.display()));
        if content.contains("crate::foundation::invariants") {
            offenders.push(path.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "contract must not depend on foundation invariants:\n{}",
        offenders.join("\n")
    );
}
