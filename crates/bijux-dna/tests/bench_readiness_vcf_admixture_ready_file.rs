#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_admixture_ready_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-admixture-ready"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/vcf/admixture-ready.json");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF admixture readiness report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_admixture_ready.v1")
    );
    assert_eq!(report.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let rows = report.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 1);

    let observed_tools = rows
        .iter()
        .filter_map(|row| row.get("tool_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(observed_tools, BTreeSet::from(["plink2"]));

    assert!(rows.iter().all(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.admixture")
            && row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("complete")
            && row.get("smoke_selected_k").and_then(serde_json::Value::as_u64) == Some(2)
            && row.get("smoke_sample_count").and_then(serde_json::Value::as_u64) == Some(4)
            && row.get("smoke_population_count").and_then(serde_json::Value::as_u64) == Some(2)
            && row
                .get("smoke_input_vcf_path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.contains("staging-"))
            && row
                .get("smoke_output_root")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value == "runs/bench/local-smoke/vcf.admixture/plink2")
    }));
}
