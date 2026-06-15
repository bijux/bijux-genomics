#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_all_retained_tools_complete_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-all-retained-tools-complete"])
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
    assert_eq!(
        rendered_path.trim(),
        "benchmarks/readiness/vcf/VCF_ALL_RETAINED_TOOLS_COMPLETE.json"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF retained-tools completion report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_all_retained_tools_complete.v1")
    );
    assert_eq!(report.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(report.get("passed_goal_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let goal_ids = report
        .get("checks")
        .and_then(serde_json::Value::as_array)
        .expect("checks array")
        .iter()
        .filter_map(|check| check.get("goal_id").and_then(serde_json::Value::as_u64))
        .collect::<BTreeSet<_>>();
    assert_eq!(goal_ids, (336_u64..=359_u64).collect::<BTreeSet<_>>());

    assert_eq!(
        report.get("local_smoke_container_row_count").and_then(serde_json::Value::as_u64),
        Some(25)
    );
}
