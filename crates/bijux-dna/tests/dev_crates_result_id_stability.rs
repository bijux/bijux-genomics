#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn dev_crates_result_id_stability_reports_canonical_alignment() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let temp = tempfile::tempdir().expect("tempdir");
    let out = temp.path().join("result-id-stability.json");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .arg("dev")
        .arg("crates")
        .arg("result-id-stability")
        .arg("--output")
        .arg(&out)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout json payload");
    let expected_output_path = out.display().to_string();
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.crates.result_id_stability.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some(expected_output_path.as_str())
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("micro_checked_row_count").and_then(serde_json::Value::as_u64), Some(3));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let vcf_call = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
        })
        .expect("vcf call row");

    for field in [
        "local_result_id",
        "fake_result_id",
        "micro_result_id",
        "report_result_id",
        "slurm_result_id",
    ] {
        assert_eq!(
            vcf_call.get(field).and_then(serde_json::Value::as_str),
            Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools"),
            "{field} must preserve the canonical result id"
        );
    }

    let local_execution_argv = vcf_call
        .get("local_execution_argv")
        .and_then(serde_json::Value::as_array)
        .expect("local argv");
    assert_eq!(
        local_execution_argv.last().and_then(serde_json::Value::as_str),
        Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
    );
    assert!(out.is_file(), "result-id stability report must be written");
}
