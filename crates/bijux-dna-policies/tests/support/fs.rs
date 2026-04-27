#![allow(non_snake_case, dead_code)]
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

pub fn crate_roots() -> Vec<PathBuf> {
    let root = workspace_root().join("crates");
    let mut crates = Vec::new();
    for entry in WalkDir::new(root).max_depth(2).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() && entry.file_name() == "Cargo.toml" {
            if let Some(parent) = entry.path().parent() {
                crates.push(parent.to_path_buf());
            }
        }
    }
    crates.sort();
    crates
}

pub fn crate_root(crate_name: &str) -> PathBuf {
    workspace_root().join("crates").join(crate_name)
}

pub fn read_to_string(path: &Path) -> String {
    bijux_dna_testkit::read_policy_text(path)
}

pub fn registry_status_is_production(status: &str) -> bool {
    matches!(status.trim(), "supported" | "production")
}
