#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

const FILE_BANS: &[&str] = &["helpers.rs", "utils.rs", "misc.rs", "support.rs", "core.rs"];
const ALLOWLIST: &[(&str, &str)] = &[];
const DIR_BANS: &[&str] = &["helpers", "util", "utils", "misc"];

#[test]
fn ban_helpers_and_utils_names() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in WalkDir::new(crate_root)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            let name = entry.file_name().to_string_lossy();
            if entry.file_type().is_file() && FILE_BANS.contains(&name.as_ref()) {
                let path_str = entry.path().to_string_lossy();
                if ALLOWLIST
                    .iter()
                    .any(|(allowed, _reason)| path_str.contains(allowed))
                {
                    continue;
                }
                offenders.push(entry.path().display().to_string());
            }
            if entry.file_type().is_dir() && DIR_BANS.contains(&name.as_ref()) {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "helpers/utils/misc/support/core names are forbidden.\n\
How to fix: rename to responsibility-specific modules (e.g. paths.rs, mounts.rs, format.rs).\n\
If truly required, add a scoped allowlist entry in tests/surface/no_helpers_policy.rs with a reason.\n\
See docs/STYLE.md for naming rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
