#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn dev_crates_graph_writes_the_governed_workspace_dependency_map() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let temp = tempfile::tempdir().expect("tempdir");
    let out = temp.path().join("crate-dependency-map.json");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .arg("dev")
        .arg("crates")
        .arg("graph")
        .arg("--output")
        .arg(&out)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout json payload");
    let expected_output_path = out.display().to_string();
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.crates.dependency_map.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some(expected_output_path.as_str())
    );
    assert!(
        payload
            .get("crate_count")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 0),
        "crate graph must report at least one workspace member"
    );
    let edges = payload.get("edges").and_then(serde_json::Value::as_array).expect("edges array");
    assert!(
        edges.iter().any(|edge| {
            edge.get("from").and_then(serde_json::Value::as_str) == Some("bijux-dna")
                && edge.get("to").and_then(serde_json::Value::as_str) == Some("bijux-dna-api")
        }),
        "workspace graph must retain the bijux-dna -> bijux-dna-api edge"
    );
    assert!(out.is_file(), "graph must write the governed report file");
}
