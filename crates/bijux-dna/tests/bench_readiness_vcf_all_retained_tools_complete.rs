#![allow(clippy::expect_used)]

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
        .args(["bench", "readiness", "render-vcf-all-retained-tools-complete", "--json"])
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
fn bench_readiness_vcf_all_retained_tools_complete_reports_governed_pass_state() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_all_retained_tools_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf/VCF_ALL_RETAINED_TOOLS_COMPLETE.json")
    );
    assert_eq!(payload.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("passed_goal_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("failing_goal_ids").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );
    assert_eq!(payload.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(44));
    assert_eq!(payload.get("retained_stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("retained_tool_count").and_then(serde_json::Value::as_u64), Some(17));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("removed_row_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("active_stage_count").and_then(serde_json::Value::as_u64), Some(17));
    assert_eq!(payload.get("active_tool_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(
        payload.get("expected_result_row_count").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(
        payload.get("rendered_command_row_count").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(
        payload.get("parser_fixture_row_count").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(payload.get("local_smoke_row_count").and_then(serde_json::Value::as_u64), Some(44));
    assert_eq!(
        payload.get("local_smoke_host_stage_row_count").and_then(serde_json::Value::as_u64),
        Some(19)
    );
    assert_eq!(
        payload.get("local_smoke_container_row_count").and_then(serde_json::Value::as_u64),
        Some(25)
    );
    assert_eq!(payload.get("report_map_row_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("ok"), Some(&serde_json::Value::Bool(true)));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    assert_eq!(checks.len(), 24);
    assert!(checks.iter().all(|check| check.get("ok") == Some(&serde_json::Value::Bool(true))));

    let goal_347 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(347))
        .expect("goal 347 check");
    assert!(
        goal_347.get("detail").and_then(serde_json::Value::as_str).is_some_and(|detail| detail
            .contains("shapeit5 active")
            && detail.contains("beagle/eagle")),
        "goal 347 detail must keep the retained phasing family explicit"
    );

    let goal_356 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(356))
        .expect("goal 356 check");
    assert!(
        goal_356
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("host-vs-container smoke")),
        "goal 356 detail must keep the local/container smoke split explicit"
    );
}
