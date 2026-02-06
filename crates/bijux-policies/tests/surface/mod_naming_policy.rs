#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn legacy_dirs_require_scope_docs() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in WalkDir::new(&crate_root)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_dir())
        {
            if entry.file_name() != "legacy" {
                continue;
            }
            let scope = entry.path().join("SCOPE.md");
            if !scope.exists() {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "legacy/ directories must include SCOPE.md:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn internal_rs_is_forbidden() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in WalkDir::new(&crate_root)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
        {
            if entry.file_name() == "internal.rs" {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "internal.rs is forbidden; use internal/mod.rs:\n{}",
        offenders.join("\n")
    );
}
