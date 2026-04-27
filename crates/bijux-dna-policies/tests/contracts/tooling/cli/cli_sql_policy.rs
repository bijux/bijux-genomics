#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__cli_sql_policy__cli_has_no_direct_sql_or_db_driver_usage() {
    let root = repo_root();
    let cli_src = root.join("crates").join("bijux-dna").join("src");
    let deny = ["rusqlite", "sqlx", "SELECT ", "INSERT ", "UPDATE ", "DELETE "];
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&cli_src)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        if deny.iter().any(|needle| content.contains(needle)) {
            offenders.push(entry.path().display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "CLI must not embed SQL/DB-driver usage; DB access belongs behind API/analyze:\n{}",
        offenders.join("\n")
    );
}
