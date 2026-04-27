#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

use support::workspace_root;

const EXCLUDE_DIRS: &[&str] = &[".git", "target", "artifacts", "site", "node_modules"];

fn is_excluded(path: &std::path::Path) -> bool {
    path.components().any(|component| {
        component.as_os_str().to_str().is_some_and(|name| EXCLUDE_DIRS.contains(&name))
    })
}

#[test]
fn slow__policy__contracts__no_appledouble__no_appledouble_artifacts() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&root) {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if is_excluded(path) {
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with("._") {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "AppleDouble files are forbidden in the repo:\n{}",
        offenders.join("\n")
    );
}
