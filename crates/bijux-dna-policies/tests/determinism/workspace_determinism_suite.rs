#![allow(non_snake_case)]
use walkdir::WalkDir;

fn workspace_root() -> std::path::PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__determinism__workspace_determinism_suite__tests_layout_by_intent_is_complete() {
    let root = workspace_root();
    let suite_buckets = ["boundaries", "contracts", "determinism", "schemas"];
    let mut offenders = Vec::new();

    let crates_dir = root.join("crates");
    for entry in std::fs::read_dir(&crates_dir)
        .unwrap_or_else(|err| panic!("read {}: {err}", crates_dir.display()))
    {
        let Ok(entry) = entry else { continue };
        let member_dir = entry.path();
        if !member_dir.is_dir() {
            continue;
        }
        let tests = member_dir.join("tests");
        if !tests.exists() {
            continue;
        }
        if !member_dir.join("docs/TESTS.md").exists() {
            offenders.push(format!("{}: missing docs/TESTS.md", member_dir.display()));
        }
        if tests.join("README.md").exists() {
            offenders.push(format!(
                "{}: test taxonomy belongs in docs/TESTS.md, not tests/README.md",
                member_dir.display()
            ));
        }
        for bucket in suite_buckets {
            let suite_dir = tests.join(bucket);
            if !suite_dir.exists() {
                continue;
            }
            let entrypoint = tests.join(format!("{bucket}.rs"));
            if !entrypoint.exists() {
                offenders.push(format!(
                    "{}: tests/{bucket}/ exists without tests/{bucket}.rs suite entrypoint",
                    member_dir.display()
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "workspace test taxonomy must use docs/TESTS.md and materialized suite entrypoints:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__determinism__workspace_determinism_suite__policy_tests_use_repo_relative_paths() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let tests_root = root.join("crates/bijux-dna-policies/tests");
    let isolate_path_marker = ["artifacts", "isolates", ""].join("/");
    let iso_tag_marker = format!("{}_{}", "ISO", "TAG");
    for entry in WalkDir::new(&tests_root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if raw.contains(&isolate_path_marker) || raw.contains(&iso_tag_marker) {
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
fn policy__determinism__workspace_determinism_suite__six_flake_snapshot_tests_remain_explicit() {
    let root = workspace_root();
    let file = root.join("crates/bijux-dna-analyze/tests/contracts/pipeline/pipeline_e2e.rs");
    let raw = std::fs::read_to_string(&file)
        .unwrap_or_else(|err| panic!("read {}: {err}", file.display()));
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
