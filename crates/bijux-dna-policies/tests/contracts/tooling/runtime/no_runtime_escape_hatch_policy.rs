#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__contracts__no_runtime_escape_hatch_policy__forbidden_terms_blocked_in_source() {
    let root = support::workspace_root();
    let forbidden = [
        "over".to_string() + "ride",
        "by".to_string() + "pass",
        "ALLOW_OVERRIDE".to_string(),
    ];
    let mut offenders = Vec::new();

    let source_roots = [
        root.join("crates/bijux-dna-cli/src"),
        root.join("crates/bijux-dna-cli/src/bin"),
    ];
    for source_root in source_roots {
        if !source_root.exists() {
            continue;
        }
        for entry in WalkDir::new(&source_root)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let path_s = path.to_string_lossy();
            if path_s.contains("/tests/dev-only/") {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if !matches!(ext, "rs" | "sh" | "py" | "bash" | "zsh") {
                continue;
            }
            let content = std::fs::read_to_string(path)
                .unwrap_or_default()
                .to_lowercase();
            if forbidden
                .iter()
                .any(|needle| content.contains(&needle.to_lowercase()))
            {
                offenders.push(path.display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "runtime escape-hatch terms are forbidden in source (override/bypass/ALLOW_OVERRIDE):\n{}",
        offenders.join("\n")
    );
}
