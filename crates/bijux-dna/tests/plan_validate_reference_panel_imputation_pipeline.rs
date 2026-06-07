#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn plan_validate_reference_panel_imputation_pipeline_reports_governed_profile() {
    let payload = run_cli_json(&[
        "plan",
        "validate",
        "--id",
        "reference-panel-imputation",
        "--strict",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_pipeline_dag_validation.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/configs/pipelines/local/reference-panel-imputation.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/reference-panel-imputation.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("reference-panel-imputation")
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));

    let profiles = payload
        .get("validation_profiles")
        .and_then(serde_json::Value::as_array)
        .expect("validation profiles");
    let panel_workflow = profiles
        .iter()
        .find(|profile| {
            profile.get("profile_id").and_then(serde_json::Value::as_str)
                == Some("reference_panel_imputation")
        })
        .expect("reference-panel imputation profile");
    assert_eq!(panel_workflow.get("check_count").and_then(serde_json::Value::as_u64), Some(8));
    assert!(
        panel_workflow
            .get("checks")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|checks| {
                checks.iter().any(|value| {
                    value.as_str()
                        == Some("vcf.impute consumes both the qc-target fallback path and phased run path with explicit panel identity")
                })
            }),
        "plan validate must surface the governed reference-panel imputation checks"
    );
}
