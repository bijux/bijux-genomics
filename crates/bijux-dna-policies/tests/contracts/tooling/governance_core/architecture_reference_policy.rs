#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

use regex::Regex;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| bijux_dna_policies::policy_panic!("resolve repository root"))
        .to_path_buf()
}

#[test]
fn policy__contracts__architecture_reference_policy__architecture_policy_paths_exist() {
    let root = repo_root();
    let docs_root = root.join("docs/10-architecture");
    let path_re = Regex::new(r"`(crates/bijux-dna-policies/tests/[^`]+\.rs)`")
        .unwrap_or_else(|err| bijux_dna_policies::policy_panic!("compile path regex: {err}"));
    let mut missing = Vec::new();

    for entry in walkdir::WalkDir::new(&docs_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("md"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_else(|err| {
            bijux_dna_policies::policy_panic!("read {}: {err}", entry.path().display())
        });
        for capture in path_re.captures_iter(&content) {
            let rel = capture.get(1).map_or("", |matched| matched.as_str());
            if !root.join(rel).is_file() {
                missing.push(format!("{} references {rel}", entry.path().display()));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "architecture docs reference missing policy test files:\n{}",
        missing.join("\n")
    );
}
