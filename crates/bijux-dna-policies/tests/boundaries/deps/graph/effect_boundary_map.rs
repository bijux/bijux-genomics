#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

const ALLOWLIST: &[(&str, &str)] = &[
    ("/crates/bijux-dna-runner/", "execution backends"),
    ("/crates/bijux-dna-environment-qa/", "qa harness"),
    ("/crates/bijux-dna/", "cli entrypoints"),
    ("/crates/bijux-dna-environment/", "runtime resolution probes"),
    ("/crates/bijux-dna-infra/", "filesystem helpers"),
    (
        "/crates/bijux-dna-core/src/foundation/input_assessment.rs",
        "documented FASTQ input assessment contract",
    ),
    ("/crates/bijux-dna-api/src/run.rs", "root CLI orchestration bridge"),
    (
        "/crates/bijux-dna-stages-vcf/",
        "transitional realization path while runtime effects are being extracted",
    ),
    (
        "/crates/bijux-dna-dev/",
        "versioned development automation owns repository-scoped process and filesystem effects",
    ),
];

const EFFECT_PATTERNS: &[&str] = &[
    "std::process::Command",
    "Command::new",
    "std::process::Stdio",
    "std::fs::remove_file",
    "std::fs::remove_dir",
    "std::fs::remove_dir_all",
];

#[test]
fn policy__boundaries__effect_boundary_map__effect_boundary_map() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let path_str = entry.path().to_string_lossy();
        if path_str.contains("/tests/") {
            continue;
        }
        if ALLOWLIST.iter().any(|(allowed, _reason)| path_str.contains(allowed)) {
            continue;
        }
        let content = support::read_to_string(entry.path());
        if EFFECT_PATTERNS.iter().any(|pattern| content.contains(pattern)) {
            offenders.push(entry.path().display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Effect boundary violation: process/Docker/FS effects are only allowed in boundary crates.\n\
Fix by moving effects into bijux-dna-runner or bijux-dna-environment-qa, or add a narrow allowlist with a reason.\n\
See docs/40-policies/STYLE.md for boundary rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
