#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .map(walkdir::DirEntry::into_path)
        .collect()
}

#[test]
fn policy__boundaries__deep_imports__api_and_cli_avoid_internal_imports() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let allowlist = [
        "crates/bijux-dna-api/src/v1/bench/exports.rs",
        "crates/bijux-dna-api/src/v1/run/entrypoints.rs",
        "crates/bijux-dna-api/src/v1/fastq/domain.rs",
    ];
    let crates = [
        root.join("crates").join("bijux-dna-api").join("src"),
        root.join("crates").join("bijux-dna").join("src"),
    ];
    for krate in crates {
        for file in collect_rs_files(&krate) {
            let path_str = file.to_string_lossy();
            if path_str.contains("/src/internal/") {
                continue;
            }
            let rel = file
                .strip_prefix(&root)
                .unwrap_or(file.as_path())
                .to_string_lossy()
                .replace('\\', "/");
            if allowlist.iter().any(|allowed| rel == *allowed) {
                continue;
            }
            let content = std::fs::read_to_string(&file).expect("read source");
            if content.contains("::internal::") {
                offenders.push(file.display().to_string());
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "API/CLI must not import internal modules: {:?}",
        offenders
    );
}
