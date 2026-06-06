#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_essential_pipeline_failure_isolation_writes_report_and_failed_tree() {
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
        .args(["bench", "readiness", "render-essential-pipeline-failure-isolation"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        rendered_path.trim(),
        "target/bench-readiness/essential-pipeline-failure-isolation.json"
    );

    let report_path = repo_root.join(rendered_path.trim());
    assert!(report_path.is_file(), "failure-isolation report must exist");
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read report"))
            .expect("parse report");
    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.essential_pipeline_failure_isolation.v1")
    );
    assert_eq!(report.get("passes_behavior_test"), Some(&serde_json::Value::Bool(true)));

    let simulation_root = repo_root.join(
        report.get("simulation_root").and_then(serde_json::Value::as_str).expect("simulation_root"),
    );
    assert!(simulation_root.join("manifest.json").is_file(), "simulation root manifest must exist");

    let failed_stage_result =
        simulation_root.join("relatedness-segments-vcf/vcf.ibd/stage-result.json");
    assert!(failed_stage_result.is_file(), "failed stage-result must exist");
    let failed_payload: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&failed_stage_result).expect("read failed stage-result"),
    )
    .expect("parse failed stage-result");
    assert_eq!(
        failed_payload
            .get("runtime")
            .and_then(|value| value.get("status"))
            .and_then(serde_json::Value::as_str),
        Some("failed")
    );
    assert_eq!(
        failed_payload
            .get("runtime")
            .and_then(|value| value.get("exit_code"))
            .and_then(serde_json::Value::as_i64),
        Some(17)
    );
    assert!(failed_payload.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| outputs
            .iter()
            .all(|output| { output.get("exists") == Some(&serde_json::Value::Bool(false)) })
    ));

    let removed_output_path =
        simulation_root.join("relatedness-segments-vcf/vcf.ibd/declared-outputs/ibd_segments.txt");
    assert!(!removed_output_path.exists(), "failed node realized outputs must be removed");

    let unrelated_stage_result =
        simulation_root.join("relatedness-segments-vcf/vcf.roh/stage-result.json");
    assert!(unrelated_stage_result.is_file(), "unrelated branch stage-result must exist");
}
