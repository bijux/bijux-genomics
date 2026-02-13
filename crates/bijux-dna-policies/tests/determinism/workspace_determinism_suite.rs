#![allow(non_snake_case)]
use std::path::Path;

use walkdir::WalkDir;

fn workspace_root() -> std::path::PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__determinism__workspace_determinism__tests_layout_by_intent_is_complete() {
    let root = workspace_root();
    let required = ["boundaries", "contracts", "determinism", "schemas"];
    let mut offenders = Vec::new();

    for entry in std::fs::read_dir(root.join("crates")).expect("read crates") {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let crate_dir = entry.path();
        if !crate_dir.is_dir() {
            continue;
        }
        let tests = crate_dir.join("tests");
        if !tests.exists() {
            continue;
        }
        if !tests.join("README.md").exists() {
            offenders.push(format!("{}: missing tests/README.md", crate_dir.display()));
        }
        for req in required {
            if !tests.join(req).exists() {
                offenders.push(format!("{}: missing tests/{req}", crate_dir.display()));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "workspace test taxonomy must be complete:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__determinism__workspace_determinism__policy_tests_use_repo_relative_paths() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let tests_root = root.join("crates/bijux-dna-policies/tests");
    for entry in WalkDir::new(&tests_root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if raw.contains("artifacts/isolates/") || raw.contains("ISO_TAG") {
            offenders.push(path.strip_prefix(&root).unwrap_or(path).display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "policy tests must not depend on isolate-tag-specific paths:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__determinism__workspace_determinism__six_flake_snapshot_tests_remain_explicit() {
    let root = workspace_root();
    let file = root.join("crates/bijux-dna-analyze/tests/contracts/pipeline/pipeline_e2e.rs");
    let raw = std::fs::read_to_string(&file).expect("read pipeline_e2e");
    let required = [
        "pipeline_fastq_to_bam_default_report_snapshot",
        "pipeline_bam_shotgun_report_snapshot",
        "pipeline_bam_capture_report_snapshot",
        "Nondeterminism vectors eliminated:",
        "Per-test unique temp roots via `TestPaths`",
        "Explicit stage sorting before report assembly",
    ];
    let mut missing = Vec::new();
    for needle in required {
        if !raw.contains(needle) {
            missing.push(needle);
        }
    }
    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "flake lock contract drifted; missing markers: {missing:?}"
    );
}
