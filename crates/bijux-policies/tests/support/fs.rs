use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

pub fn crate_roots() -> Vec<PathBuf> {
    let root = workspace_root().join("crates");
    let mut crates = Vec::new();
    for entry in WalkDir::new(root)
        .max_depth(2)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
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
    std::fs::read_to_string(path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", path.display());
    })
}
