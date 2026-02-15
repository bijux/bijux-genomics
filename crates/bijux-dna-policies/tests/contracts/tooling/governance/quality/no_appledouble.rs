#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

use support::workspace_root;

const EXCLUDE_DIRS: &[&str] = &[".git", "target", "artifacts", "site", "node_modules"];

fn is_excluded(path: &std::path::Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .map(|name| EXCLUDE_DIRS.contains(&name))
            .unwrap_or(false)
    })
}

#[test]
fn policy__contracts__no_appledouble__no_appledouble_or_ds_store() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&root) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if is_excluded(path) {
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if name == ".DS_Store" || name.starts_with("._") {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "AppleDouble/DS_Store files are forbidden in the repo:\n{}",
        offenders.join("\n")
    );
}
