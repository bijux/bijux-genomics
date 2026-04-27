#![allow(non_snake_case)]
#[path = "../support/fs.rs"]
mod support;

use std::path::Path;

use walkdir::WalkDir;

const MAX_FIXTURE_BYTES: u64 = 200 * 1024;

fn is_allowlisted(path: &Path) -> bool {
    let allowlist = ["tests/fixtures/golden_spine"]; // large reference fixtures
    allowlist.iter().any(|prefix| path.to_string_lossy().contains(prefix))
}

#[test]
fn policy__boundaries__fixtures_policy__fixture_lint() {
    let mut offenders = Vec::new();

    for crate_root in support::crate_roots() {
        let fixtures_root = crate_root.join("tests").join("fixtures");
        if !fixtures_root.exists() {
            continue;
        }

        for entry in WalkDir::new(&fixtures_root).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if entry.file_type().is_file() {
                if path.parent() == Some(fixtures_root.as_path()) {
                    offenders.push(format!("orphan fixture file at root: {}", path.display()));
                }
                if !is_allowlisted(path) {
                    if let Ok(meta) = path.metadata() {
                        if meta.len() > MAX_FIXTURE_BYTES {
                            offenders.push(format!(
                                "fixture too large (>{} bytes): {}",
                                MAX_FIXTURE_BYTES,
                                path.display()
                            ));
                        }
                    }
                }
            }
        }

        for entry in WalkDir::new(&fixtures_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_dir())
        {
            let dir = entry.path();
            let has_subdir = dir
                .read_dir()
                .ok()
                .is_some_and(|mut it| it.any(|e| e.ok().is_some_and(|e| e.path().is_dir())));
            let has_case = dir.join("CASE.toml").exists() || dir.join("CASE.json").exists();
            if !has_subdir && !has_case {
                offenders
                    .push(format!("missing CASE.(toml|json) in fixture dir: {}", dir.display()));
            }
            if dir.join("CASE.md").exists() {
                offenders.push(format!(
                    "legacy CASE.md is forbidden; use CASE.toml/json: {}",
                    dir.display()
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "fixture lint failed. Fix fixture structure and CASE.(toml|json) coverage.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
