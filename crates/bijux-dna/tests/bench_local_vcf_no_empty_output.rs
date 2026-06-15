#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_local_validate_vcf_no_empty_output_reports_governed_outputs() {
    let payload = run_cli_json(&["bench", "local", "validate-vcf-no-empty-output", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_no_empty_output_check.v1")
    );
    assert_eq!(
        payload.get("report_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/vcf/no-empty-output-check.json")
    );
    assert_eq!(
        payload.get("smoke_root_manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf/SMOKE_ROOT.json")
    );
    assert_eq!(
        payload.get("smoke_root_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf")
    );
    assert_eq!(
        payload.get("refreshed_smoke_outputs").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("tool_pair_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("checked_output_count").and_then(serde_json::Value::as_u64), Some(61));
    assert_eq!(payload.get("non_empty_output_count").and_then(serde_json::Value::as_u64), Some(61));
    assert_eq!(payload.get("empty_output_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("missing_output_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("allowed_empty_output_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 61);

    let prepare_panel_vcf = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.prepare_reference_panel")
                && row.get("output_id").and_then(serde_json::Value::as_str)
                    == Some("prepared_panel")
        })
        .expect("prepare panel VCF row");
    assert_eq!(
        prepare_panel_vcf.get("output_kind").and_then(serde_json::Value::as_str),
        Some("vcf")
    );
    assert_eq!(
        prepare_panel_vcf.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf/vcf.prepare_reference_panel/bcftools/artifacts/prepared_panel.vcf.gz")
    );
    assert!(prepare_panel_vcf
        .get("bytes")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|bytes| bytes > 0));
    assert_eq!(
        prepare_panel_vcf.get("status").and_then(serde_json::Value::as_str),
        Some("non_empty")
    );

    let ibd_segments = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.ibd")
                && row.get("output_id").and_then(serde_json::Value::as_str) == Some("ibd_segments")
        })
        .expect("IBD segments row");
    assert_eq!(ibd_segments.get("output_kind").and_then(serde_json::Value::as_str), Some("tsv"));
    assert_eq!(ibd_segments.get("status").and_then(serde_json::Value::as_str), Some("non_empty"));

    let stderr_log = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats")
                && row.get("output_id").and_then(serde_json::Value::as_str) == Some("stderr_log")
        })
        .expect("stderr log row");
    assert_eq!(stderr_log.get("output_kind").and_then(serde_json::Value::as_str), Some("log"));
    assert_eq!(
        stderr_log.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf/vcf.stats/bcftools/artifacts/stderr.log")
    );
}
