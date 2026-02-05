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

#[test]
fn workflows_use_tools_scripts_only() {
    let root = workspace_root();
    let workflows_dir = root.join(".github").join("workflows");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(workflows_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("yml") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read workflow");
        if content.contains("cargo clippy")
            || content.contains("cargo fmt")
            || content.contains("cargo test")
            || content.contains("cargo nextest")
            || content.contains("cargo make")
        {
            offenders.push(entry.path().display().to_string());
        }
        if !content.contains("tools/") {
            offenders.push(entry.path().display().to_string());
        }
    }
    offenders.sort();
    offenders.dedup();
    assert!(
        offenders.is_empty(),
        "CI workflows must use tools/* entrypoints only: {:?}",
        offenders
    );
}
