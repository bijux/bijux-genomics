#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use support::{crate_roots, read_to_string, workspace_root};

fn docs_root() -> PathBuf {
    workspace_root().join("docs")
}

fn is_uppercase_stem(path: &Path) -> bool {
    path.file_stem().and_then(|stem| stem.to_str()).is_some_and(|stem| stem == stem.to_uppercase())
}

#[test]
fn policy__boundaries__docs_spine__docs_placement_contract() {
    let root = docs_root();
    let allowed_root_dirs = BTreeSet::from([
        "00-intro",
        "10-architecture",
        "20-science",
        "30-operations",
        "40-policies",
        "50-reference",
        "decisions",
        "assets",
        "cli",
        "containers",
        "overrides",
    ]);
    let allowed_root_files = BTreeSet::from(["index.md", "DOCS_GRAPH.toml", "badges.md"]);
    let mut root_entries = Vec::new();
    for entry in std::fs::read_dir(&root).expect("read docs root") {
        let entry = entry.expect("read entry");
        let name = entry.file_name().to_string_lossy().to_string();
        root_entries.push(name.clone());
        let path = entry.path();
        if path.is_dir() {
            if !allowed_root_dirs.contains(name.as_str()) {
                bijux_dna_policies::policy_panic!(
                    "docs root contains unexpected directory: {}",
                    name
                );
            }
        } else if path.is_file() && !allowed_root_files.contains(name.as_str()) {
            bijux_dna_policies::policy_panic!("docs root contains unexpected file: {}", name);
        }
    }
    bijux_dna_policies::policy_assert!(root_entries.contains(&"index.md".to_string()));
}

#[test]
fn policy__boundaries__docs_spine__no_docs_under_src() {
    for crate_root in crate_roots() {
        let src = crate_root.join("src");
        if !src.exists() {
            continue;
        }
        for entry in WalkDir::new(&src) {
            let entry = entry.expect("walk src");
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|ext| ext.to_str()) == Some("md")
                && !matches!(
                    entry.path().file_name().and_then(|name| name.to_str()),
                    Some("OWNER.md" | "INDEX.md")
                )
            {
                bijux_dna_policies::policy_panic!(
                    "docs under src are forbidden: {}",
                    entry.path().display()
                );
            }
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__crate_docs_contract() {
    for crate_root in crate_roots() {
        let docs = crate_root.join("docs");
        let readme = crate_root.join("README.md");
        if !readme.exists() {
            bijux_dna_policies::policy_panic!("crate README.md missing: {}", crate_root.display());
        }
        for entry in std::fs::read_dir(&crate_root).expect("read crate root") {
            let entry = entry.expect("read entry");
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if file_name != "README.md"
                && file_name != "BOUNDARY.md"
                && file_name != "PUBLIC_API.md"
            {
                bijux_dna_policies::policy_panic!(
                    "crate root contains extra doc: {}",
                    path.display()
                );
            }
        }
        let mut required = BTreeSet::from(["ARCHITECTURE.md"]);
        if !docs.exists() {
            bijux_dna_policies::policy_panic!("crate docs directory missing: {}", docs.display());
        }
        for entry in std::fs::read_dir(&docs).expect("read docs dir") {
            let entry = entry.expect("read entry");
            let name = entry.file_name().to_string_lossy().to_string();
            if !is_uppercase_stem(entry.path().as_path()) {
                bijux_dna_policies::policy_panic!(
                    "crate docs filename must be uppercase: {}",
                    entry.path().display()
                );
            }
            required.remove(name.as_str());
        }
        if !required.is_empty() {
            bijux_dna_policies::policy_panic!(
                "crate docs missing required architecture files in {}: {:?}",
                docs.display(),
                required
            );
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__root_docs_style_template() {
    let root = docs_root();
    let roots = [
        root.join("00-intro"),
        root.join("10-architecture"),
        root.join("20-science"),
        root.join("30-operations"),
        root.join("40-policies"),
        root.join("50-reference"),
    ];
    for dir in roots {
        for entry in WalkDir::new(&dir) {
            let entry = entry.expect("walk docs");
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if entry
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".generated.md"))
            {
                continue;
            }
            if entry.path().starts_with(root.join("20-science")) {
                continue;
            }
            if entry.path().starts_with(root.join("30-operations"))
                || entry.path().starts_with(root.join("50-reference"))
            {
                continue;
            }
            let filename = entry.path().file_name().and_then(|name| name.to_str());
            if filename != Some("index.md") && !is_uppercase_stem(entry.path()) {
                bijux_dna_policies::policy_panic!(
                    "root docs filename must be uppercase: {}",
                    entry.path().display()
                );
            }
            if filename == Some("index.md") {
                continue;
            }
            let _ = read_to_string(entry.path());
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__root_docs_metadata_headers() {
    let root = docs_root();
    let key_docs = [
        root.join("10-architecture/SSOT.md"),
        root.join("10-architecture/BOUNDARY_MAP.md"),
        root.join("10-architecture/CONTRACT_SPINE.md"),
    ];
    let required =
        ["Owner:", "Scope:", "Last reviewed:", "Contract version:", "Applies to crates:"];
    for doc in key_docs {
        let content = read_to_string(&doc);
        for header in required {
            if !content.lines().any(|line| line.starts_with(header)) {
                bijux_dna_policies::policy_panic!(
                    "missing metadata header {header} in {}",
                    doc.display()
                );
            }
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__stage_catalog_schema() {
    let root = docs_root();
    let catalogs = [
        root.join("20-science/fastq/STAGE_CATALOG.md"),
        root.join("20-science/bam/STAGE_CATALOG.md"),
    ];
    let required =
        ["Purpose:", "Inputs/Outputs:", "Metrics:", "Tools:", "Defaults:", "References:"];
    for catalog in catalogs {
        let content = read_to_string(&catalog);
        for section in content.split("\n### ").skip(1) {
            for field in required {
                if !section.contains(field) {
                    bijux_dna_policies::policy_panic!(
                        "stage catalog missing field {field} in {}",
                        catalog.display()
                    );
                }
            }
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__authority_docs_unique_h1() {
    let root = docs_root();
    let key_docs = [
        root.join("10-architecture/SSOT.md"),
        root.join("10-architecture/BOUNDARY_MAP.md"),
        root.join("10-architecture/CONTRACT_SPINE.md"),
    ];
    let mut seen = HashMap::new();
    for doc in key_docs {
        let content = read_to_string(&doc);
        let title = content.lines().find(|line| line.starts_with("# ")).unwrap_or("");
        if let Some(prev) = seen.insert(title.to_string(), doc.clone()) {
            bijux_dna_policies::policy_panic!(
                "duplicate H1 heading {title} in {} and {}",
                prev.display(),
                doc.display()
            );
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__contract_versioning_note_exists() {
    let doc = docs_root().join("50-reference/CONTRACT_VERSIONING.md");
    let content = read_to_string(&doc);
    let required = ["Breaking change", "major bump", "minor bump", "snapshot"];
    for phrase in required {
        if !content.contains(phrase) {
            bijux_dna_policies::policy_panic!("contract versioning doc missing phrase {phrase}");
        }
    }
}
