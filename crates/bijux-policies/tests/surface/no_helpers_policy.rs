#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

const FILE_BANS: &[&str] = &["helpers.rs", "utils.rs", "misc.rs"];
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
                offenders.push(entry.path().display().to_string());
            }
            if entry.file_type().is_dir() && DIR_BANS.contains(&name.as_ref()) {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "helpers/utils/misc names are forbidden:\n{}",
        offenders.join("\n")
    );
}
