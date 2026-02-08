#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use support::crate_roots;

const MAX_FLAT_TEST_FILES: usize = 20;
const ALLOWLIST: &[(&str, &str)] = &[];

fn is_allowlisted(crate_name: &str) -> bool {
    ALLOWLIST.iter().any(|(name, _)| *name == crate_name)
}

#[test]
fn policy__boundaries__test_grouping_policy__tests_are_grouped_into_subsuites() {
    let mut offenders = Vec::new();

    for crate_root in crate_roots() {
        let crate_name = crate_root.file_name().unwrap().to_string_lossy();
        let tests_dir = crate_root.join("tests");
        if !tests_dir.exists() {
            continue;
        }
        let count = std::fs::read_dir(&tests_dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs")
                    })
                    .count()
            })
            .unwrap_or(0);
        if count > MAX_FLAT_TEST_FILES && !is_allowlisted(&crate_name) {
            offenders.push(format!("{} ({} files)", crate_name, count));
        }
    }

    if !offenders.is_empty() {
        let allowlist_hint = ALLOWLIST
            .iter()
            .map(|(name, reason)| format!("- {name}: {reason}"))
            .collect::<Vec<_>>()
            .join("\n");
        bijux_policies::policy_panic!(
            "Crates must group tests into sub-suites when tests/ grows too large.\n\
MAX = {MAX_FLAT_TEST_FILES} flat files.\n\
Offenders:\n{}\n\nAllowlist (temporary):\n{}",
            offenders.join("\n"),
            if allowlist_hint.is_empty() {
                "(none)"
            } else {
                &allowlist_hint
            }
        );
    }
}
