#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

const ALLOWLIST: &[&str] = &[
    "/crates/bijux-runner/",
    "/crates/bijux-environment-qa/",
    "/crates/bijux-cli/",
];

const EFFECT_PATTERNS: &[&str] = &[
    "std::process::Command",
    "Command::new",
    "std::fs::remove_file",
    "std::fs::remove_dir",
    "std::fs::remove_dir_all",
];

#[test]
fn effect_boundary_map() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
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
        if ALLOWLIST.iter().any(|allowed| path_str.contains(allowed)) {
            continue;
        }
        let content = support::read_to_string(entry.path());
        if EFFECT_PATTERNS.iter().any(|pattern| content.contains(pattern)) {
            offenders.push(entry.path().display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "Effect boundary violation: process/FS effects are only allowed in runner/QA/CLI.\n\
Fix by moving effects into bijux-runner or bijux-environment-qa, or add a narrow allowlist.\n\
See STYLE.md for boundary rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
