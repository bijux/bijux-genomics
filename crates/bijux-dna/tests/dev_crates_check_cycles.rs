#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn dev_crates_check_cycles_writes_the_governed_audit_report() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let temp = tempfile::tempdir().expect("tempdir");
    let out = temp.path().join("no-crate-cycles.json");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .arg("dev")
        .arg("crates")
        .arg("check-cycles")
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
        Some("bijux.crates.no_crate_cycles.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some(expected_output_path.as_str())
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("cycle_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("audited_crate_count").and_then(serde_json::Value::as_u64), Some(16));
    assert_eq!(payload.get("category_count").and_then(serde_json::Value::as_u64), Some(7));

    let categories =
        payload.get("categories").and_then(serde_json::Value::as_array).expect("categories array");
    let parser = categories
        .iter()
        .find(|row| row.get("category").and_then(serde_json::Value::as_str) == Some("parser"))
        .expect("parser category report");
    assert_eq!(parser.get("crate_count").and_then(serde_json::Value::as_u64), Some(0));

    let crates = payload.get("crates").and_then(serde_json::Value::as_array).expect("crates array");
    let bench_model = crates
        .iter()
        .find(|crate_report| {
            crate_report.get("crate_name").and_then(serde_json::Value::as_str)
                == Some("bijux-dna-bench-model")
        })
        .expect("bench model report");
    assert_eq!(bench_model.get("category").and_then(serde_json::Value::as_str), Some("benchmark"));
    let bench_model_deps = bench_model
        .get("direct_workspace_dependencies")
        .and_then(serde_json::Value::as_array)
        .expect("direct_workspace_dependencies");
    assert!(
        bench_model_deps
            .iter()
            .all(|dependency| { dependency.as_str() != Some("bijux-dna-analyze") }),
        "bench model audit must retain the removed analyze dependency edge"
    );

    let topo = payload
        .get("topological_order")
        .and_then(serde_json::Value::as_array)
        .expect("topological_order");
    let topo_names = topo
        .iter()
        .map(|row| row.as_str().expect("topological order entry").to_string())
        .collect::<Vec<_>>();
    let cli_index = topo_names
        .iter()
        .position(|crate_name| crate_name == "bijux-dna")
        .expect("topological order must include bijux-dna");
    let dev_index = topo_names
        .iter()
        .position(|crate_name| crate_name == "bijux-dna-dev")
        .expect("topological order must include bijux-dna-dev");
    assert!(dev_index < cli_index, "dependency order must list bijux-dna-dev before bijux-dna");
    assert_eq!(topo_names.len(), 16, "topological order must cover the full audited crate set");
    assert!(out.is_file(), "audit must write the governed report file");
}
