#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__error_boundary_policy__domain_crates_must_not_reference_runner_errors() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let path_str = path.display().to_string();
        if !path_str.contains("bijux-dna-domain-") {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if raw.contains("bijux_dna_runner")
            || raw.contains("RunnerEffectKind")
            || raw.contains("[runner_effect:")
        {
            offenders.push(path_str);
        }
    }
    assert!(
        offenders.is_empty(),
        "domain crates must stay pure and must not reference runner error types:\n{}",
        offenders.join("\n")
    );
}
