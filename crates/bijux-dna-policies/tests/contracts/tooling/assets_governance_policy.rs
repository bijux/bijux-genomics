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
        } else if name != "index.md" && name != "CONTRACT.md" && name != "LARGE_FILE_ALLOWLIST.txt" {
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
#[ignore = "TODO: align publication manifest requirements with current assets/publications contract"]
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
        let manifest = path.join("MANIFEST.toml");
        if !manifest.is_file() {
            offenders.push(format!("assets/publications/{id}/MANIFEST.toml missing"));
            continue;
        }
        let raw = std::fs::read_to_string(&manifest).unwrap_or_default();
        let parsed: toml::Value = match toml::from_str(&raw) {
            Ok(v) => v,
            Err(err) => {
                offenders.push(format!(
                    "assets/publications/{id}/MANIFEST.toml invalid TOML: {err}"
                ));
                continue;
            }
        };
        for required in ["title", "authors", "year", "provenance_notes", "license"] {
            if parsed.get(required).is_none() {
                offenders.push(format!(
                    "assets/publications/{id}/MANIFEST.toml missing field `{required}`"
                ));
            }
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
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if name == "GENERATE.md" || name.ends_with(".md") {
            continue;
        }
        let Some(dir) = path.parent() else {
            continue;
        };
        if !dir.join("GENERATE.md").is_file() {
            offenders.push(
                path.strip_prefix(&root)
                    .unwrap_or(path)
                    .display()
                    .to_string(),
            );
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "golden files must declare deterministic generation metadata:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__toy_datasets_require_checksums() {
    let root = repo_root();
    let toy = root.join("assets/toy");
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&toy).expect("read toy dir") {
        let entry = entry.expect("read toy entry");
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if !path.join("CHECKSUMS.sha256").is_file() {
            offenders.push(format!("assets/toy/{id}/CHECKSUMS.sha256 missing"));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "toy datasets must include CHECKSUMS.sha256:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__assets_forbid_local_machine_paths() {
    let root = repo_root();
    let assets = root.join("assets");
    let mut offenders = Vec::new();
    let banned = ["/Users/", "/home/", "C:\\\\Users\\\\", "\\\\Users\\\\"];
    for entry in WalkDir::new(&assets)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !matches!(
            ext,
            "yaml" | "yml" | "json" | "jsonl" | "toml" | "txt" | "md" | "vcf" | "sam" | "fastq"
        ) {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if banned.iter().any(|needle| raw.contains(needle)) {
            offenders.push(
                path.strip_prefix(&root)
                    .unwrap_or(path)
                    .display()
                    .to_string(),
            );
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "assets must not embed local-machine/PII path literals:\\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__tests_must_not_write_into_assets() {
    let root = repo_root();
    let mut offenders = Vec::new();
    let write_markers = [
        "write(",
        "create(",
        "create_dir(",
        "create_dir_all(",
        "OpenOptions",
        "remove_file(",
        "remove_dir(",
        "rename(",
        "copy(",
    ];
    for dir in ["crates", "scripts", "makefiles"] {
        for entry in WalkDir::new(root.join(dir))
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let rel = path.strip_prefix(&root).unwrap_or(path);
            let rel_s = rel.to_string_lossy();
            if rel_s
                == "crates/bijux-dna-policies/tests/contracts/tooling/assets_governance_policy.rs"
            {
                continue;
            }
            let is_testish = rel_s.contains("/tests/")
                || rel_s.starts_with("scripts/test/")
                || rel_s.ends_with(".mk");
            if !is_testish {
                continue;
            }
            let raw = std::fs::read_to_string(path).unwrap_or_default();
            if raw.contains("assets/") && write_markers.iter().any(|marker| raw.contains(marker)) {
                offenders.push(rel.display().to_string());
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tests/tooling must not write into assets/ (assets are read-only):\\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__docs_publication_refs_use_publications_prefix() {
    let root = repo_root();
    let docs = root.join("docs");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&docs)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        for token in raw.split_whitespace() {
            if let Some(idx) = token.find("assets/publications/") {
                let frag = &token[idx..];
                if !frag.contains("/index.md") {
                    offenders.push(format!(
                        "{}: publication refs must target assets/publications/<pub-id>/index.md",
                        path.strip_prefix(&root).unwrap_or(path).display()
                    ));
                    break;
                }
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "docs asset references must use assets/publications/<pub-id>/...:\\n{}",
        offenders.join("\n")
    );
}
