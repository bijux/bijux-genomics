#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__boundaries__mod_naming_policy__legacy_dirs_require_legacy_doc() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in WalkDir::new(&crate_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_dir())
        {
            if entry.file_name() != "legacy" {
                continue;
            }
            let legacy_doc = crate_root.join("docs").join("LEGACY.md");
            if !legacy_doc.exists() {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "legacy/ directories must include docs/LEGACY.md at crate root:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__mod_naming_policy__internal_rs_is_forbidden() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in WalkDir::new(&crate_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            if entry.file_name() == "internal.rs" {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "internal.rs is forbidden; use internal/mod.rs:\n{}",
        offenders.join("\n")
    );
}
