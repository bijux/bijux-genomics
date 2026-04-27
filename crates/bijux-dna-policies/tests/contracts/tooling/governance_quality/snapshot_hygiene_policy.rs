#![allow(non_snake_case)]
use std::path::PathBuf;

use regex::Regex;
use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__snapshot_hygiene_policy__snapshots_avoid_hostnames_and_wallclock_timestamps()
{
    let root = repo_root();
    let snapshots_root = root.join("crates");
    let host_or_uri =
        Regex::new("(?i)\\b(localhost|127\\.0\\.0\\.1)\\b|https?://[^\\s\"'<>()]+").expect("regex");
    let wallclock = Regex::new(
        r"\b20[0-9]{2}-[01][0-9]-[0-3][0-9]T[0-2][0-9]:[0-5][0-9]:[0-5][0-9](\.[0-9]+)?Z\b",
    )
    .expect("regex");
    let home_or_user = Regex::new(
        r"(/Users/|/home/|\\Users\\|\\home\\|/private/var/|C:\\Users\\|(?i)\b(user(name)?)\b)",
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
        if path_s.ends_with(
            "crates/bijux-dna-pipelines/tests/snapshots/bijux-dna-pipelines__contracts__fastq_reference_adna_profile.snap",
        ) || path_s.ends_with(
            "crates/bijux-dna-pipelines/tests/snapshots/bijux-dna-pipelines__contracts__defaults__fastq-to-fastq__reference_adna__v1.snap",
        ) {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        if host_or_uri.is_match(&content)
            || wallclock.is_match(&content)
            || home_or_user.is_match(&content)
        {
            offenders.push(path.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "snapshots must be deterministic and must not include hostnames/wallclock timestamps:\n{}",
        offenders.join("\n")
    );
}
