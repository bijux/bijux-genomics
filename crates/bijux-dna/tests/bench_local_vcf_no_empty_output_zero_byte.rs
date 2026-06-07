#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

#[test]
fn bench_local_validate_vcf_no_empty_output_rejects_zero_byte_artifact() {
    let initial = run_cli(&["bench", "local", "validate-vcf-no-empty-output", "--json"]);
    assert!(
        initial.status.success(),
        "initial command failed: {}\nstdout:\n{}\nstderr:\n{}",
        initial.status,
        String::from_utf8_lossy(&initial.stdout),
        String::from_utf8_lossy(&initial.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&initial.stdout).expect("parse initial report");
    let target_path = payload
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .expect("rows array")
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats")
                && row.get("output_id").and_then(serde_json::Value::as_str) == Some("stats_json")
        })
        .and_then(|row| row.get("output_path").and_then(serde_json::Value::as_str))
        .expect("stats_json output path");

    let repo_root = support::repo_root().expect("repo root");
    std::fs::write(repo_root.join(target_path), []).expect("truncate stats_json");

    let failing =
        run_cli(&["bench", "local", "validate-vcf-no-empty-output", "--skip-refresh", "--json"]);
    assert!(
        !failing.status.success(),
        "zero-byte output should fail validation\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&failing.stdout),
        String::from_utf8_lossy(&failing.stderr)
    );

    let stderr = String::from_utf8_lossy(&failing.stderr);
    assert!(stderr.contains("vcf.stats"), "failure should name the stage: {stderr}");
    assert!(stderr.contains("stats_json"), "failure should name the output id: {stderr}");
    assert!(stderr.contains("status `empty`"), "failure should identify empty status: {stderr}");

    let report_path =
        repo_root.join("benchmarks/readiness/local-ready/vcf/no-empty-output-check.json");
    let raw = std::fs::read_to_string(report_path).expect("read failure report");
    let report: serde_json::Value = serde_json::from_str(&raw).expect("parse failure report");
    assert_eq!(report.get("valid").and_then(serde_json::Value::as_bool), Some(false));
    assert_eq!(report.get("empty_output_count").and_then(serde_json::Value::as_u64), Some(1));
    assert!(
        report.get("missing_output_count").and_then(serde_json::Value::as_u64).unwrap_or(0) == 0
    );
}
