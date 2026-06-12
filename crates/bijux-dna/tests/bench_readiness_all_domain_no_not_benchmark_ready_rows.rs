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
fn bench_readiness_all_domain_no_not_benchmark_ready_rows_reports_clean_active_scope() {
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-all-domain-no-not-benchmark-ready-rows",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_no_not_benchmark_ready_rows.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/no-not-benchmark-ready-rows.json")
    );
    let executable_active_row_count = support::json_u64(&payload, "executable_active_row_count")
        .expect("executable_active_row_count");
    let active_row_count = support::json_u64(&payload, "active_row_count").expect("active_row_count");
    let removed_row_count = support::json_u64(&payload, "removed_row_count").expect("removed_row_count");
    assert_eq!(executable_active_row_count, active_row_count + removed_row_count);
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(131));
    assert_eq!(payload.get("active_stage_count").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(payload.get("active_tool_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(removed_row_count, 6);
    assert_eq!(payload.get("removed_stage_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("removed_tool_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let removed_status_counts = payload
        .get("removed_status_counts")
        .and_then(serde_json::Value::as_object)
        .expect("removed status counts");
    assert_eq!(
        removed_status_counts.get("not_benchmark_ready").and_then(serde_json::Value::as_u64),
        Some(6)
    );

    let removed_rows = support::json_array(&payload, "removed_rows");
    assert_eq!(removed_rows.len() as u64, removed_row_count);
    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(violations.is_empty(), "active scope must not retain not_benchmark_ready rows");

    assert!(removed_rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.index_reference")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2_build")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("not_benchmark_ready")
    }));
    assert!(removed_rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.profile_overrepresented_sequences")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqkit")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("not_benchmark_ready")
    }));
    assert!(removed_rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("multiqc")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("not_benchmark_ready")
    }));
}
