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
fn bench_readiness_all_domain_no_planned_rows_reports_clean_active_scope() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-no-planned-rows", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_no_planned_rows.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/no-planned-rows.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(126));
    assert_eq!(payload.get("active_stage_count").and_then(serde_json::Value::as_u64), Some(59));
    assert_eq!(payload.get("active_tool_count").and_then(serde_json::Value::as_u64), Some(67));
    assert_eq!(payload.get("removed_row_count").and_then(serde_json::Value::as_u64), Some(13));
    assert_eq!(payload.get("removed_stage_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(payload.get("removed_tool_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let removed_status_counts = payload
        .get("removed_status_counts")
        .and_then(serde_json::Value::as_object)
        .expect("removed status counts");
    assert_eq!(removed_status_counts.get("planned").and_then(serde_json::Value::as_u64), Some(13));

    let removed_rows =
        payload.get("removed_rows").and_then(serde_json::Value::as_array).expect("removed rows");
    assert_eq!(removed_rows.len(), 13);
    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(violations.is_empty(), "active scope must not retain planned rows");

    assert!(removed_rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.estimate_library_complexity_prealign")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("planned")
    }));

    assert!(
        removed_rows.iter().all(|row| {
            !(row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("vcf.phasing")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit5"))
        }),
        "vcf.phasing/shapeit5 must remain active once phasing is benchmark ready"
    );
}
