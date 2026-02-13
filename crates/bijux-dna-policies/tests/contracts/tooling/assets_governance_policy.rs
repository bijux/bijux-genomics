#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("resolve repo root")
        .to_path_buf()
}

#[test]
fn policy__contracts__assets_governance_policy__assets_root_uses_taxonomy_dirs_only() {
    let root = repo_root();
    let assets = root.join("assets");
    let mut offenders = Vec::new();
    let allowed_dirs = ["publications", "golden", "toy", "reference"];
    for entry in std::fs::read_dir(&assets).expect("read assets dir") {
        let entry = entry.expect("read assets entry");
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if !allowed_dirs.contains(&name.as_str()) {
                offenders.push(format!("unexpected directory: assets/{name}"));
            }
        } else if name != "index.md" {
            offenders.push(format!("unexpected file: assets/{name}"));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "assets root must contain only taxonomy dirs + index.md:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__publication_dirs_require_manifest_toml() {
    let root = repo_root();
    let publications = root.join("assets/publications");
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&publications).expect("read publications dir") {
        let entry = entry.expect("read publication entry");
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if id == "index.md" {
            continue;
        }
        if !path.join("MANIFEST.toml").is_file() {
            offenders.push(format!("assets/publications/{id}/MANIFEST.toml missing"));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "publication asset dirs must include MANIFEST.toml:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__golden_files_require_generate_metadata() {
    let root = repo_root();
    let golden = root.join("assets/golden");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&golden)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        if name == "GENERATE.md"
            || name == "GENERATE.toml"
            || name.ends_with(".md")
        {
            continue;
        }
        let Some(dir) = path.parent() else {
            continue;
        };
        if !(dir.join("GENERATE.md").is_file() || dir.join("GENERATE.toml").is_file()) {
            offenders.push(path.strip_prefix(&root).unwrap_or(path).display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "golden files must declare deterministic generation metadata:\n{}",
        offenders.join("\n")
    );
}
