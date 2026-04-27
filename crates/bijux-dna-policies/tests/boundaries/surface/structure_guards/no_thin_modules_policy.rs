#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::path::Path;

use walkdir::WalkDir;

const ALLOWLIST_DIRS: &[(&str, &str)] = &[
    ("prelude", "explicit reexport surface"),
    (
        "crates/bijux-dna-core/src/public_api/metrics",
        "public_api is intentionally segmented by stable surface area",
    ),
    (
        "crates/bijux-dna-core/src/public_api/identity",
        "public_api is intentionally segmented by stable surface area",
    ),
    (
        "crates/bijux-dna-core/src/public_api/contracts",
        "public_api is intentionally segmented by stable surface area",
    ),
    (
        "crates/bijux-dna-core/src/public_api/catalog",
        "public_api is intentionally segmented by stable surface area",
    ),
    (
        "crates/bijux-dna-core/src/public_api/ergonomics",
        "public_api is intentionally segmented by stable surface area",
    ),
    (
        "crates/bijux-dna-db-ena/src/public_api",
        "public_api entrypoint is intentionally isolated under dedicated namespace",
    ),
    (
        "crates/bijux-dna-db-ref/src/public_api",
        "public_api entrypoint is intentionally isolated under dedicated namespace",
    ),
    (
        "crates/bijux-dna-engine/src/executor/graph",
        "graph module anchors nested topology and contract submodules",
    ),
];
const MIN_PUB_ITEMS: usize = 5;

fn is_allowlisted_dir(path: &Path) -> bool {
    let path_str = path.to_string_lossy().replace('\\', "/");
    ALLOWLIST_DIRS.iter().any(|(name, _reason)| path.ends_with(name) || path_str.ends_with(name))
}

fn pub_item_count(content: &str) -> usize {
    content.matches("pub ").count()
}

#[test]
fn policy__boundaries__no_thin_modules_policy__no_thin_module_directories() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        let src_dir = crate_root.join("src");
        if !src_dir.exists() {
            continue;
        }
        for entry in WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_dir())
        {
            let dir = entry.path();
            if is_allowlisted_dir(dir) {
                continue;
            }
            let mut rs_files = Vec::new();
            let mut has_other_entries = false;
            if let Ok(read_dir) = std::fs::read_dir(dir) {
                for child in read_dir.flatten() {
                    let path = child.path();
                    if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                        rs_files.push(path);
                    } else {
                        has_other_entries = true;
                    }
                }
            }
            if has_other_entries || rs_files.len() >= 2 {
                continue;
            }
            if rs_files.len() == 1
                && rs_files[0].file_name().and_then(|n| n.to_str()) == Some("mod.rs")
            {
                let content = support::read_to_string(&rs_files[0]);
                if pub_item_count(&content) >= MIN_PUB_ITEMS {
                    continue;
                }
                offenders.push(dir.display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "directories with only mod.rs must be expanded or collapsed.\n\
How to fix: collapse the directory to a single .rs file or add real submodules.\n\
If a thin module is intentional, add it to ALLOWLIST_DIRS with a reason.\n\
See docs/40-policies/STYLE.md for module rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
