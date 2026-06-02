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

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
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
fn bench_readiness_undercovered_stages_reports_single_backend_gaps() {
    let payload = run_cli_json(&["bench", "readiness", "render-undercovered-stages", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.undercovered_stages.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/undercovered-stages.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(
        payload.get("undercovered_stage_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(false));

    let domain_counts = payload
        .get("domain_counts")
        .and_then(serde_json::Value::as_object)
        .expect("domain_counts object");
    assert_eq!(
        domain_counts.get("bam").and_then(serde_json::Value::as_u64),
        Some(1),
        "the current undercovered-stage slice must be entirely BAM-owned"
    );
    assert!(
        domain_counts.get("fastq").is_none(),
        "FASTQ currently has no undercovered benchmark stages in this governed slice"
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 1, "the governed undercovered-stage slice must retain one BAM row");
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("bam.overlap_correction")
                && row.get("registered_tool_ids").and_then(serde_json::Value::as_array)
                    == Some(&vec![serde_json::Value::String("bamutil".to_string())])
                && row.get("missing_tool_ids").and_then(serde_json::Value::as_array)
                    == Some(&vec![serde_json::Value::String("samtools".to_string())])
        }),
        "bam.overlap_correction must stay visible as a single-backend benchmark gap"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.filter")
        }),
        "bam.filter must stay out of the undercovered-stage report once all admitted tools are registered"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.mapq_filter")
        }),
        "bam.mapq_filter must stay out of the undercovered-stage report once all admitted tools are registered"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.length_filter")
        }),
        "bam.length_filter must stay out of the undercovered-stage report once all admitted tools are registered"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.markdup")
        }),
        "bam.markdup must stay out of the undercovered-stage report once all admitted tools are registered"
    );
}
