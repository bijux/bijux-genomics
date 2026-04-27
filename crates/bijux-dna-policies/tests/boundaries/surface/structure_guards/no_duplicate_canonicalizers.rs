#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__boundaries__no_duplicate_canonicalizers__core_is_only_canonicalizer() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let path_str = entry.path().to_string_lossy();
        if path_str.contains("/crates/bijux-dna-core/") {
            continue;
        }
        let content = support::read_to_string(entry.path());
        if content.contains(concat!("fn ", "canonicalize_json_value"))
            || content.contains(concat!("fn ", "normalize_numbers_and_paths"))
            || content.contains(concat!("fn ", "to_canonical_json_bytes"))
        {
            offenders.push(entry.path().display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Canonicalization must live in bijux-dna-core only.\n\
Use bijux_dna_core::contract::canonical instead of re-implementing.\n\
See docs/40-policies/STYLE.md.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
