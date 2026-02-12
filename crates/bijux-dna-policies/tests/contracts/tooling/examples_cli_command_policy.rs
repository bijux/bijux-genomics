#![allow(non_snake_case)]
use std::path::Path;

#[test]
fn policy__contracts__examples_cli_command_policy__examples_use_existing_cli_commands_only() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let examples = root.join("examples");
    let allowed = [
        "run", "plan", "explain", "analyze", "bench", "env", "registry", "corpus", "status",
    ];

    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&examples)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if !matches!(ext, "sh" | "md" | "txt") {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.contains("bijux ") {
                continue;
            }
            if trimmed.contains("bijux example") || trimmed.contains("bijux dna example") {
                offenders.push(format!(
                    "{}: legacy example command is forbidden: {}",
                    path.display(),
                    trimmed
                ));
                continue;
            }
            if let Some(idx) = trimmed.find("bijux dna ") {
                let tail = &trimmed[idx + "bijux dna ".len()..];
                let verb = tail.split_whitespace().next().unwrap_or_default();
                if !allowed.contains(&verb) {
                    offenders.push(format!(
                        "{}: unknown/non-allowlisted verb `{}` in line: {}",
                        path.display(),
                        verb,
                        trimmed
                    ));
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "examples must use existing CLI commands only:\n{}",
        offenders.join("\n")
    );
}
