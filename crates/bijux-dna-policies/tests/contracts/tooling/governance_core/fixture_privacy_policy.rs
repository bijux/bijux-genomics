#![allow(non_snake_case)]
use std::path::PathBuf;

use regex::Regex;
use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__fixture_privacy_policy__fixtures_do_not_embed_absolute_host_paths() {
    let root = repo_root();
    let fixtures_root = root.join("crates");
    let absolute_path = Regex::new(r"(/Users/|/home/|[A-Za-z]:\\)").expect("regex");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&fixtures_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let path_s = path.to_string_lossy();
        if !path_s.contains("/tests/fixtures/") {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        if absolute_path.is_match(&content) {
            offenders.push(path.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "fixtures must not contain host absolute paths:\n{}",
        offenders.join("\n")
    );
}
