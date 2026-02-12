#![allow(non_snake_case)]
use std::path::Path;

#[test]
fn policy__contracts__cli_runtime_dependency_policy__cli_has_no_sql_or_process_dependencies() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let manifest_path = root.join("crates/bijux-dna-cli/Cargo.toml");
    let manifest =
        std::fs::read_to_string(&manifest_path).expect("read crates/bijux-dna-cli/Cargo.toml");

    for forbidden in ["rusqlite", "tokio-process", "tokio ="] {
        bijux_dna_policies::policy_assert!(
            !manifest.contains(forbidden),
            "cli manifest must not depend on {forbidden}: {}",
            manifest_path.display()
        );
    }
}

#[test]
fn policy__contracts__cli_runtime_dependency_policy__cli_source_does_not_spawn_processes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let src = root.join("crates/bijux-dna-cli/src");
    let mut offenders = Vec::new();

    for entry in walkdir::WalkDir::new(&src).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read cli source");
        if content.contains("std::process::Command")
            || content.contains("tokio::process::Command")
            || content.contains("Command::new(")
        {
            offenders.push(entry.path().display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "cli source must not spawn processes directly (move behind api/runtime):\n{}",
        offenders.join("\n")
    );
}
