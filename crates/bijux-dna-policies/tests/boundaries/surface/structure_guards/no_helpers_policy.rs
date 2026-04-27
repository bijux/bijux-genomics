#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

const FILE_BANS: &[&str] = &["helpers.rs", "utils.rs", "misc.rs", "support.rs", "core.rs"];
const ALLOWLIST: &[(&str, &str)] = &[
    (
        "/crates/bijux-dna-api/src/internal/fastq/stages/correct_errors/support.rs",
        "stage-scoped support surface keeps correction wiring isolated",
    ),
    (
        "/crates/bijux-dna-core/src/id_catalog/stage/core.rs",
        "core is a canonical domain identifier namespace",
    ),
    (
        "/crates/bijux-dna-domain-fastq/src/observer/contracts/core.rs",
        "observer contract core module names stable interface primitives",
    ),
    (
        "/crates/bijux-dna-stage-contract/src/execution_plan/support.rs",
        "execution-plan support module provides contract-only adapters",
    ),
];
const DIR_BANS: &[&str] = &["helpers", "util", "utils", "misc"];

#[test]
fn policy__boundaries__no_helpers_policy__ban_helpers_and_utils_names() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in WalkDir::new(crate_root).into_iter().filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy();
            if entry.file_type().is_file() && FILE_BANS.contains(&name.as_ref()) {
                let path_str = entry.path().to_string_lossy();
                if ALLOWLIST.iter().any(|(allowed, _reason)| path_str.contains(allowed)) {
                    continue;
                }
                offenders.push(entry.path().display().to_string());
            }
            if entry.file_type().is_dir() && DIR_BANS.contains(&name.as_ref()) {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "helpers/utils/misc/support/core names are forbidden.\n\
How to fix: rename to responsibility-specific modules (e.g. paths.rs, mounts.rs, format.rs).\n\
If truly required, add a scoped allowlist entry in tests/surface/no_helpers_policy.rs with a reason.\n\
See docs/40-policies/STYLE.md for naming rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
