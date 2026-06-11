#![allow(clippy::expect_used)]

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
        .args(["bench", "readiness", "render-operational-benchmark-ready", "--json"])
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
fn bench_readiness_operational_benchmark_ready_reports_green_surface() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.operational_benchmark_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/FASTQ_BAM_VCF_OPERATIONAL_BENCHMARK_READY.json")
    );
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(127)
    );
    assert_eq!(payload.get("blocker_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("missing_result_row_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        payload.get("insufficient_data_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("unsupported_pair_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    let blockers =
        payload.get("blockers").and_then(serde_json::Value::as_array).expect("blockers array");
    assert!(checks.iter().all(|check| check.get("ok") == Some(&serde_json::Value::Bool(true))));
    assert!(blockers.is_empty(), "green operational gate must not emit blockers");

    let surface_ids = checks
        .iter()
        .filter_map(|check| check.get("surface_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    for required_surface in [
        "fastq_bam_benchmark_binding_coverage",
        "vcf_stage_catalog_ready",
        "vcf_smoke_suite_ready",
        "stage_tool_resources",
        "full_benchmark_result_collector",
        "full_benchmark_report",
        "full_benchmark_dashboard",
    ] {
        assert!(
            surface_ids.contains(required_surface),
            "operational gate must keep `{required_surface}` explicit"
        );
    }
}
