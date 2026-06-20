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
    assert_eq!(payload.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(19));
    assert_eq!(payload.get("passed_goal_count").and_then(serde_json::Value::as_u64), Some(19));
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
        payload.get("output_declaration_row_count").and_then(serde_json::Value::as_u64),
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
    let expected_host_stage_smoke_row_count =
        if cfg!(feature = "bam_downstream") { 20 } else { 18 };
    let expected_container_smoke_row_count = if cfg!(feature = "bam_downstream") { 29 } else { 31 };
    assert_eq!(
        payload.get("local_smoke_host_stage_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_host_stage_smoke_row_count)
    );
    assert_eq!(
        payload.get("local_smoke_container_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_container_smoke_row_count)
    );
    assert_eq!(payload.get("report_map_row_count").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(
        payload.get("active_row_consistency_surface_count").and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(
        payload.get("micro_smoke_family_count").and_then(serde_json::Value::as_u64),
        Some(12)
    );
    assert_eq!(
        payload.get("science_threshold_stage_count").and_then(serde_json::Value::as_u64),
        Some(15)
    );
    assert_eq!(payload.get("ok"), Some(&serde_json::Value::Bool(true)));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    assert_eq!(checks.len(), 19);
    assert!(checks.iter().all(|check| check.get("ok") == Some(&serde_json::Value::Bool(true))));

    let goal_439 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(439))
        .expect("goal 439 check");
    assert!(
        goal_439
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("bam.genotyping")),
        "goal 439 detail must keep BAM genotyping explicit"
    );

    let goal_448 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(448))
        .expect("goal 448 check");
    assert!(
        goal_448
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("stage family")),
        "goal 448 detail must keep the BAM stage-family micro smoke proof explicit"
    );
}
