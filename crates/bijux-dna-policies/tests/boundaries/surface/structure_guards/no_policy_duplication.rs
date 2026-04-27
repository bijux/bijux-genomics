#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__boundaries__no_policy_duplication__policies_live_only_in_bijux_dna_policies() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.to_string_lossy().contains("/crates/bijux-dna-policies/") {
            continue;
        }
        if !path.to_string_lossy().contains("/tests/") {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.contains("policy") {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Policy-like tests must live in bijux-dna-policies only.\n\
Move policy scans into bijux-dna-policies and keep other crates to fixtures only.\n\
See docs/40-policies/STYLE.md for governance rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
