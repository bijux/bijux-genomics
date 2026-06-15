#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

fn is_policy_like(name: &str) -> bool {
    name.contains("guardrails")
        || name.contains("boundary")
        || name.contains("no_process_spawn")
        || name.contains("no_runner_usage")
        || name.contains("no_execution")
        || name.contains("no_spawn")
        || name.contains("purity")
}

#[test]
fn policy__boundaries__no_duplicate_policy_checks__no_duplicate_policy_checks() {
    let mut offenders = Vec::new();

    for crate_root in support::crate_roots() {
        if crate_root.file_name().and_then(|name| name.to_str()) == Some("bijux-dna-policies") {
            continue;
        }
        let tests_root = crate_root.join("tests");
        if !tests_root.exists() {
            continue;
        }
        for entry in WalkDir::new(&tests_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("rs"))
        {
            let path = entry.path();
            if path.to_string_lossy().contains("tests/boundaries")
                || path.to_string_lossy().contains("tests/guardrails.rs")
            {
                continue;
            }
            if is_policy_like(&path.file_name().unwrap().to_string_lossy()) {
                offenders.push(path.display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "policy-like tests must live in bijux-dna-policies or boundaries suites only.\n\
Move duplicated policy checks to bijux-dna-policies.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
