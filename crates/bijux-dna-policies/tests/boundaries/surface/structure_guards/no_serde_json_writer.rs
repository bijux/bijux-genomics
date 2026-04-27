#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__boundaries__no_serde_json_writer__serde_json_to_writer_is_banned() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    let patterns =
        [concat!("serde_json::", "to_writer"), concat!("serde_json::", "to_writer_pretty")];
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let path_str = entry.path().to_string_lossy();
        if path_str.contains("/crates/bijux-dna-core/src/contract/canonical/") {
            continue;
        }
        let content = support::read_to_string(entry.path());
        if patterns.iter().any(|pattern| content.contains(pattern)) {
            offenders.push(entry.path().display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Contract JSON must be written via core canonicalizer, not serde_json to_writer.\n\
Use bijux_dna_core::contract::canonical::to_canonical_json_bytes and bijux_dna_infra::atomic_write_bytes.\n\
See docs/40-policies/STYLE.md.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
