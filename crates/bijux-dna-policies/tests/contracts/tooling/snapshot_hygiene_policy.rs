#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("resolve repo root")
        .to_path_buf()
}

#[test]
fn policy__contracts__snapshot_hygiene_policy__snapshots_avoid_hostnames_and_wallclock_timestamps()
{
    let root = repo_root();
    let snapshots_root = root.join("crates");
    let host_or_uri =
        Regex::new(r#"(?i)\b(localhost|127\.0\.0\.1)\b|https?://[^\s"'<>()]+"#).expect("regex");
    let wallclock = Regex::new(
        r#"\b20[0-9]{2}-[01][0-9]-[0-3][0-9]T[0-2][0-9]:[0-5][0-9]:[0-5][0-9](\.[0-9]+)?Z\b"#,
    )
    .expect("regex");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&snapshots_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let path_s = path.to_string_lossy();
        if !path_s.contains("/tests/snapshots/") {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        if host_or_uri.is_match(&content) || wallclock.is_match(&content) {
            offenders.push(path.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "snapshots must be deterministic and must not include hostnames/wallclock timestamps:\n{}",
        offenders.join("\n")
    );
}
