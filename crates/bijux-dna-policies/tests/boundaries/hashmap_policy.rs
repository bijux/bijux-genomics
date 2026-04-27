#![allow(non_snake_case)]
#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__boundaries__hashmap_policy__no_hashmap_in_contract_paths() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in WalkDir::new(&crate_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            let path_str = path.to_string_lossy();
            if !path_str.contains("src/") {
                continue;
            }
            if !(path_str.contains("contract")
                || path_str.contains("schema")
                || path_str.contains("report"))
            {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let content = support::read_to_string(path);
            if content.contains("HashMap") {
                offenders.push(path.display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "HashMap is forbidden in public contract/schema/report serialization paths.\n\
Use BTreeMap or deterministic maps.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
