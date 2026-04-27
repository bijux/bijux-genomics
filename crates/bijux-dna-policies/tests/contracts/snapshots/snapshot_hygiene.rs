#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__snapshot_hygiene__no_absolute_paths_or_hostnames() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|s| s.to_str()) != Some("snap") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let bad = content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("/Users/")
                || trimmed.starts_with("\\Users\\")
                || trimmed.starts_with("/tmp/")
                || trimmed.starts_with("C:\\\\")
                || trimmed.contains("/home/")
                || trimmed.contains("\\home\\")
                || trimmed.contains("/private/var/")
                || trimmed.contains("hostname")
        });
        if bad {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "snapshots must not include absolute paths or hostnames: {offenders:?}"
    );
}
