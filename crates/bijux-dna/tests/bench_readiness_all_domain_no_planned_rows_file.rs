#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_no_planned_rows_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-all-domain-no-planned-rows"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/all-domains/no-planned-rows.json");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read all-domain no-planned-rows report");
    let payload: serde_json::Value = serde_json::from_str(&payload).expect("parse report json");

    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(126));
    assert_eq!(payload.get("removed_row_count").and_then(serde_json::Value::as_u64), Some(17));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let removed_rows =
        payload.get("removed_rows").and_then(serde_json::Value::as_array).expect("removed rows");
    assert!(removed_rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.prepare_reference_panel")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("planned")
    }));
}
