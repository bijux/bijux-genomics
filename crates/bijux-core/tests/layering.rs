use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .map(|entry| entry.into_path())
        .collect()
}

#[test]
fn primitives_do_not_depend_on_plan_or_contract() {
    let root = crate_root().join("src").join("primitives");
    let mut offenders = Vec::new();
    for path in collect_rs_files(&root) {
        let content = std::fs::read_to_string(&path).expect("read source");
        if content.contains("crate::plan") || content.contains("crate::contract") {
            offenders.push(path.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "primitives must not depend on plan/contract:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn contract_does_not_depend_on_plan() {
    let root = crate_root().join("src").join("contract");
    let mut offenders = Vec::new();
    for path in collect_rs_files(&root) {
        let content = std::fs::read_to_string(&path).expect("read source");
        if content.contains("crate::plan") {
            offenders.push(path.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "contract must not depend on plan:\n{}",
        offenders.join("\n")
    );
}
