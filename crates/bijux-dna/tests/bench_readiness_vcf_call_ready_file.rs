#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_call_ready_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-call-ready"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/vcf/call-ready.json");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF call readiness report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_call_ready.v1")
    );
    assert_eq!(report.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let row = report
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| rows.first())
        .expect("first row");
    assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.call"));
    assert_eq!(row.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(
        row.get("expected_outputs").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("called_vcf".to_string())])
    );
    assert!(row.get("report_metric_columns").and_then(serde_json::Value::as_array).is_some_and(
        |columns| {
            columns.iter().any(|value| value.as_str() == Some("variant_count"))
                && columns.iter().any(|value| value.as_str() == Some("sample_count"))
        }
    ));
    assert!(row.get("raw_outputs").and_then(serde_json::Value::as_array).is_some_and(|outputs| {
        outputs.iter().any(|value| {
            value.as_str()
                == Some(
                    "called_vcf=benchmarks/readiness/adapters/bcftools/vcf.call/called_vcf.vcf.gz",
                )
        })
    }));
    assert!(
        row.get("index_outputs")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|outputs| outputs.iter().any(|value| {
                value.as_str()
                    == Some(
                        "called_vcf_tbi=benchmarks/readiness/adapters/bcftools/vcf.call/called_vcf.vcf.gz.tbi",
                    )
            }))
    );
    assert_eq!(
        row.get("reason").and_then(serde_json::Value::as_str),
        Some(
            "retained VCF caller `bcftools` keeps active scope, command, output, parser, expected-result, report, and smoke proof for `vcf.call`",
        )
    );
}
