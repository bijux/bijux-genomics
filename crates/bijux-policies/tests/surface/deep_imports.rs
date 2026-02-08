#![allow(non_snake_case)]
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

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .map(|entry| entry.into_path())
        .collect()
}

#[test]
fn policy__surface__deep_imports__api_and_cli_avoid_internal_imports() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let crates = [
        root.join("crates").join("bijux-api").join("src"),
        root.join("crates").join("bijux-cli").join("src"),
    ];
    for krate in crates {
        for file in collect_rs_files(&krate) {
            let content = std::fs::read_to_string(&file).expect("read source");
            if content.contains("::internal::") {
                offenders.push(file.display().to_string());
            }
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "API/CLI must not import internal modules: {:?}",
        offenders
    );
}
