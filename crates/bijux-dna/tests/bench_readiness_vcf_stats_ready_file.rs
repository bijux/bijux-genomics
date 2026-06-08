#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_stats_ready_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-stats-ready"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/vcf/stats-ready.json");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF stats readiness report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_stats_ready.v1")
    );
    assert_eq!(report.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let row = report
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| rows.first())
        .expect("first row");
    assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.stats"));
    assert_eq!(row.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(
        row.get("expected_outputs").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("stats_json".to_string())])
    );
    assert!(row.get("report_metric_columns").and_then(serde_json::Value::as_array).is_some_and(
        |columns| {
            columns.iter().any(|value| value.as_str() == Some("variant_count"))
                && columns.iter().any(|value| value.as_str() == Some("transition_count"))
                && columns.iter().any(|value| value.as_str() == Some("transversion_count"))
                && columns.iter().any(|value| value.as_str() == Some("ti_tv"))
        }
    ));
    assert!(
        row.get("raw_outputs")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|outputs| outputs.iter().any(|value| {
                value.as_str()
                    == Some(
                        "bcftools_stats_txt=benchmarks/readiness/adapters/bcftools/vcf.stats/bcftools_stats.txt",
                    )
            }))
    );
    assert!(row
        .get("normalized_metrics_outputs")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| outputs.iter().any(|value| {
            value.as_str()
                == Some(
                    "stats_json=benchmarks/readiness/adapters/bcftools/vcf.stats/stats_json.json",
                )
        })));
    assert_eq!(
        row.get("reason").and_then(serde_json::Value::as_str),
        Some(
            "active retained VCF stats caller `bcftools` keeps active scope, command, output, parser, expected-result, report, and normalized stats smoke proof for `vcf.stats`",
        )
    );
}
