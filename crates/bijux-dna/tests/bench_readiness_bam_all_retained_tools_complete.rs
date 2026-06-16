#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json() -> serde_json::Value {
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
        .args(["bench", "readiness", "render-bam-all-retained-tools-complete", "--json"])
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
fn bench_readiness_bam_all_retained_tools_complete_reports_governed_pass_state() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_all_retained_tools_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/BAM_ALL_RETAINED_TOOLS_COMPLETE.json")
    );
    assert_eq!(payload.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(17));
    assert_eq!(payload.get("passed_goal_count").and_then(serde_json::Value::as_u64), Some(17));
    assert_eq!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("failing_goal_ids").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );
    assert_eq!(payload.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(payload.get("retained_stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("retained_tool_count").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(
        payload.get("command_adapter_row_count").and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(
        payload.get("expected_result_row_count").and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(
        payload.get("rendered_command_row_count").and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(
        payload.get("parser_fixture_row_count").and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(payload.get("local_smoke_row_count").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(
        payload.get("local_smoke_host_stage_row_count").and_then(serde_json::Value::as_u64),
        Some(18)
    );
    assert_eq!(
        payload.get("local_smoke_container_row_count").and_then(serde_json::Value::as_u64),
        Some(31)
    );
    assert_eq!(payload.get("report_map_row_count").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(payload.get("ok"), Some(&serde_json::Value::Bool(true)));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    assert_eq!(checks.len(), 17);
    assert!(checks.iter().all(|check| check.get("ok") == Some(&serde_json::Value::Bool(true))));

    let goal_390 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(390))
        .expect("goal 390 check");
    assert!(
        goal_390
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("recalibration and genotyping")),
        "goal 390 detail must keep the BAM recalibration/genotyping slice explicit"
    );

    let goal_392 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(392))
        .expect("goal 392 check");
    assert!(
        goal_392
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("host-vs-container smoke")),
        "goal 392 detail must keep the local/container smoke split explicit"
    );
}
