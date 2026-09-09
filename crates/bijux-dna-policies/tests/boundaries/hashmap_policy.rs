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
            if !is_contract_source(&crate_root, path) {
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

fn is_contract_source(crate_root: &std::path::Path, path: &std::path::Path) -> bool {
    let Ok(relative) = path.strip_prefix(crate_root.join("src")) else {
        return false;
    };
    let relative = relative.to_string_lossy();
    let owner = crate_root.file_name().unwrap_or_default().to_string_lossy();
    path.extension().and_then(|extension| extension.to_str()) == Some("rs")
        && [owner.as_ref(), relative.as_ref()].iter().any(|part| {
            part.contains("contract") || part.contains("schema") || part.contains("report")
        })
}

#[test]
fn policy__boundaries__hashmap_policy__contract_scope_ignores_checkout_names() {
    for checkout in ["checkout", "contract-workspace", "schema-report-workspace"] {
        let root = std::path::Path::new(checkout).join("crates/owner");
        for relative in ["src/contract/model.rs", "src/schema.rs", "src/reports/table.rs"] {
            assert!(is_contract_source(&root, &root.join(relative)), "{relative}");
        }
        for relative in ["src/runtime/map.rs", "tests/contracts/model.rs", "src/schema.md"] {
            assert!(!is_contract_source(&root, &root.join(relative)), "{relative}");
        }
        let contract_owner = std::path::Path::new(checkout).join("crates/owner-stage-contract");
        assert!(is_contract_source(&contract_owner, &contract_owner.join("src/model.rs")));
    }
}
