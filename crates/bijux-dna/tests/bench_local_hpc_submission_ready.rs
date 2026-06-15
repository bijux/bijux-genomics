#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[cfg(feature = "bam_downstream")]
use std::fs;

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

#[cfg(feature = "bam_downstream")]
fn run_hpc_submission_ready_report() -> serde_json::Value {
    let output = run_cli(&["bench", "local", "validate-hpc-submission-ready", "--json"]);

    assert!(
        !output.status.success(),
        "expected governed local HPC readiness blockers\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("local HPC submission readiness failed"),
        "stderr must report the governed readiness failure, got:\n{stderr}"
    );

    let repo_root = support::repo_root().expect("repo root");
    let report_path = repo_root.join("benchmarks/readiness/local-ready/HPC_SUBMISSION_READY.json");
    serde_json::from_slice::<serde_json::Value>(&fs::read(&report_path).expect("read report"))
        .expect("parse report")
}

#[cfg(not(feature = "bam_downstream"))]
#[test]
fn bench_local_validate_hpc_submission_ready_refuses_without_bam_downstream() {
    let output = run_cli(&["bench", "local", "validate-hpc-submission-ready", "--json"]);

    assert!(
        !output.status.success(),
        "command should fail without bam_downstream\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("requires the `bam_downstream` feature"),
        "stderr must explain the bam_downstream requirement, got:\n{stderr}"
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_validate_hpc_submission_ready_reports_governed_blockers() {
    let payload = run_hpc_submission_ready_report();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_hpc_submission_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/HPC_SUBMISSION_READY.json")
    );
    assert_eq!(payload.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(99));
    assert_eq!(payload.get("passed_goal_count").and_then(serde_json::Value::as_u64), Some(97));
    assert_eq!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(false));
    assert!(
        payload.get("failing_goal_ids").and_then(serde_json::Value::as_array).is_some_and(
            |goal_ids| {
                goal_ids == &[serde_json::Value::from(65_u64), serde_json::Value::from(66_u64)]
            }
        ),
        "governed readiness gate must report the known failing goal ids"
    );

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    assert_eq!(checks.len(), 99);
    assert!(
        checks.iter().any(|check| {
            check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(56)
                && check.get("ok").and_then(serde_json::Value::as_bool) == Some(true)
                && check.get("output_path").and_then(serde_json::Value::as_str)
                    == Some("runs/bench/local-fake-runs/stages")
        }),
        "goal 56 must report the governed fake-run output root"
    );
    assert!(
        checks.iter().any(|check| {
            check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(91)
                && check.get("output_path").and_then(serde_json::Value::as_str)
                    == Some("benchmarks/configs/hpc/campaign/cross-mini.toml")
                && check
                    .get("detail")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|detail| detail.contains("cross-hpc-mini"))
        }),
        "goal 91 must report the benchmark-owned HPC campaign config root"
    );
    assert!(
        checks.iter().any(|check| {
            check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(99)
                && check.get("output_path").and_then(serde_json::Value::as_str)
                    == Some("benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml")
                && check
                    .get("detail")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|detail| detail.contains("planned 4 jobs"))
        }),
        "goal 99 must report the benchmark-owned LUNARC local-ready profile"
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_validate_hpc_submission_ready_writes_governed_report_path() {
    let _payload = run_hpc_submission_ready_report();

    let repo_root = support::repo_root().expect("repo root");
    let report_path = repo_root.join("benchmarks/readiness/local-ready/HPC_SUBMISSION_READY.json");
    assert!(report_path.is_file(), "HPC submission readiness report must exist");

    let report =
        serde_json::from_slice::<serde_json::Value>(&fs::read(&report_path).expect("read report"))
            .expect("parse report");
    assert_eq!(
        report.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/HPC_SUBMISSION_READY.json")
    );
    assert_eq!(report.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(2));
}
