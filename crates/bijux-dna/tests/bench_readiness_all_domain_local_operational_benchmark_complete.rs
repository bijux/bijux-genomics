#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
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
        .args([
            "bench",
            "readiness",
            "render-all-domain-local-operational-benchmark-complete",
            "--json",
        ])
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
fn bench_readiness_all_domain_local_operational_benchmark_complete_reports_green_surface() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_local_operational_benchmark_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some(
            "benchmarks/readiness/all-domains/FASTQ_BAM_VCF_LOCAL_OPERATIONAL_BENCHMARK_COMPLETE.json"
        )
    );
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("failed_surface_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(140));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(140)
    );
    assert_eq!(payload.get("blocker_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    assert!(checks.iter().all(|check| check.get("ok") == Some(&serde_json::Value::Bool(true))));

    let surface_ids = checks
        .iter()
        .filter_map(|check| check.get("surface_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    for required_surface in [
        "benchmark_paths_cleanup_proof",
        "all_domain_active_scope_complete",
        "operational_benchmark_ready",
    ] {
        assert!(
            surface_ids.contains(required_surface),
            "final Goal 400 gate must keep `{required_surface}` explicit"
        );
    }
}
