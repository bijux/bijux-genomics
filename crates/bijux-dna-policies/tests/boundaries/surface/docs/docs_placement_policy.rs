#![allow(non_snake_case)]

use std::path::PathBuf;

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__boundaries__docs_placement_policy__docs_live_in_crate_docs_only() {
    let root = workspace_root();
    let crates_dir = root.join("crates");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&crates_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or_default();
        let is_readme = file_name == "README.md";
        let is_docs_path = path.components().any(|component| component.as_os_str() == "docs");
        if path.to_string_lossy().contains("/tests/fixtures/") {
            continue;
        }
        if path.to_string_lossy().contains("/tests/snapshots/") {
            continue;
        }
        if path.to_string_lossy().contains("/crates/bijux-dna-bench/bench/") {
            continue;
        }
        if path.to_string_lossy().contains("/tests/support/README.md") {
            continue;
        }
        if is_readme || file_name == "BOUNDARY.md" || file_name == "PUBLIC_API.md" {
            if let Some(parent) = path.parent() {
                if let Some(grandparent) = parent.parent() {
                    if grandparent.ends_with("crates") {
                        continue;
                    }
                }
            }
        }
        if !is_readme && !is_docs_path {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "docs must live under crates/*/docs (only README.md allowed at crate root):\n{}",
        offenders.join("\n")
    );
}
