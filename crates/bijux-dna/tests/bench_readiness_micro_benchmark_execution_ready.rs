#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json() -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _repo_lock =
        support::RepoProcessLock::acquire("benchmark-readiness-mutators").expect("repo lock");
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
        .args(["bench", "readiness", "render-micro-benchmark-execution-ready", "--json"])
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
fn bench_readiness_micro_benchmark_execution_ready_reports_green_gate() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.micro_benchmark_execution_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/micro/MICRO_BENCHMARK_EXECUTION_READY.json")
    );
    assert_eq!(payload.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("passed_goal_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("coverage_check_count").and_then(serde_json::Value::as_u64), Some(11));
    assert_eq!(
        payload.get("passed_coverage_check_count").and_then(serde_json::Value::as_u64),
        Some(11)
    );
    assert_eq!(
        payload.get("failed_coverage_check_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("result_row_count").and_then(serde_json::Value::as_u64), Some(77));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(68));
    assert_eq!(payload.get("unavailable_row_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    let coverage_checks = payload
        .get("coverage_checks")
        .and_then(serde_json::Value::as_array)
        .expect("coverage checks array");
    assert_eq!(checks.len(), 9);
    assert_eq!(coverage_checks.len(), 11);
    assert!(checks
        .iter()
        .all(|check| check.get("ok").and_then(serde_json::Value::as_bool) == Some(true)));
    assert!(coverage_checks
        .iter()
        .all(|check| check.get("ok").and_then(serde_json::Value::as_bool) == Some(true)));

    let goal_ids = checks
        .iter()
        .filter_map(|check| check.get("goal_id").and_then(serde_json::Value::as_u64))
        .collect::<BTreeSet<_>>();
    assert_eq!(goal_ids, BTreeSet::from([471, 472, 473, 474, 475, 476, 477, 478, 479]));

    let coverage_ids = coverage_checks
        .iter()
        .filter_map(|check| check.get("coverage_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    for required_coverage in [
        "report.health",
        "domain.fastq",
        "domain.bam",
        "domain.vcf",
        "family.fastq",
        "family.bam",
        "family.vcf",
        "pipeline.core_germline",
        "pipeline.adna",
        "pipeline.edna",
        "pipeline.amplicon",
    ] {
        assert!(
            coverage_ids.contains(required_coverage),
            "micro execution gate must keep `{required_coverage}` explicit"
        );
    }

    let goal_476 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(476))
        .expect("goal 476 check");
    assert!(
        goal_476
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("structured skips")),
        "goal 476 detail must keep structured aDNA skips visible"
    );
}
