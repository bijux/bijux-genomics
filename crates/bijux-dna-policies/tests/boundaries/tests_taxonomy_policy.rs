#![allow(non_snake_case)]
#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

const FOUNDATION_DNA_CRATES: &[&str] = &[
    "bijux-dna",
    "bijux-dna-api",
    "bijux-dna-core",
    "bijux-dna-dev",
    "bijux-dna-engine",
    "bijux-dna-infra",
    "bijux-dna-policies",
    "bijux-dna-runner",
    "bijux-dna-runtime",
    "bijux-dna-testkit",
];

#[test]
fn policy__boundaries__tests_taxonomy_policy__taxonomy_buckets_only() {
    let allowed = [
        "contracts",
        "schemas",
        "semantics",
        "determinism",
        "boundaries",
        "fixtures",
        "snapshots",
        "support",
    ];
    let mut offenders = Vec::new();

    for crate_name in FOUNDATION_DNA_CRATES {
        let crate_root = support::crate_root(crate_name);
        let tests_root = crate_root.join("tests");
        if !tests_root.exists() {
            continue;
        }
        for entry in WalkDir::new(&tests_root)
            .max_depth(2)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_dir())
        {
            let dir = entry.path();
            if dir == tests_root {
                continue;
            }
            let rel = dir.strip_prefix(&tests_root).unwrap();
            let name = rel.components().next().unwrap().as_os_str().to_string_lossy();
            if !allowed.contains(&name.as_ref()) {
                let readme = dir.join("README.md");
                if !readme.exists() {
                    offenders.push(dir.display().to_string());
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tests must use canonical buckets or include README.md justification.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__tests_taxonomy_policy__no_shadow_suite_aggregators() {
    let suite_names = ["boundaries", "contracts", "determinism", "schemas", "semantics"];
    let mut offenders = Vec::new();

    for crate_name in FOUNDATION_DNA_CRATES {
        let crate_root = support::crate_root(crate_name);
        let tests_root = crate_root.join("tests");
        if !tests_root.exists() {
            continue;
        }
        for suite in suite_names {
            let nested_aggregator = tests_root.join(suite).join(format!("{suite}.rs"));
            if nested_aggregator.exists() {
                offenders.push(
                    nested_aggregator
                        .strip_prefix(support::workspace_root())
                        .unwrap_or(&nested_aggregator)
                        .display()
                        .to_string(),
                );
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "suite aggregators must live at tests/<suite>.rs only; nested \
tests/<suite>/<suite>.rs files are uncompiled shadow entrypoints.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
