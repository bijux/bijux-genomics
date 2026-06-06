#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json() -> serde_json::Value {
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
        .args(["bench", "readiness", "render-vcf-adapter-missing-input-tests", "--json"])
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
fn bench_readiness_vcf_adapter_missing_input_tests_report_governed_roles() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_adapter_missing_input_tests.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/vcf-adapter-missing-input-tests.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("passed_row_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("adapter_row_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("support_row_count").and_then(serde_json::Value::as_u64), Some(1));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 10);

    for role in [
        "bam",
        "bai",
        "fasta",
        "fai",
        "vcf",
        "vcf_index",
        "sites_bed",
        "panel_vcf",
        "map_file",
        "sample_metadata",
    ] {
        assert!(
            rows.iter().any(|row| {
                row.get("missing_input_role").and_then(serde_json::Value::as_str) == Some(role)
                    && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
                    && row
                        .get("expected_error_fragment")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|fragment| fragment.contains("required input"))
            }),
            "report must retain passing missing-input coverage for {role}"
        );
    }

    let vcf_index = rows
        .iter()
        .find(|row| {
            row.get("missing_input_role").and_then(serde_json::Value::as_str) == Some("vcf_index")
        })
        .expect("vcf index row");
    assert_eq!(vcf_index.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.phasing"));
    assert_eq!(vcf_index.get("tool_id").and_then(serde_json::Value::as_str), Some("shapeit5"));
    assert_eq!(
        vcf_index.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("benchmark_ready")
    );
    assert!(
        vcf_index
            .get("artifact_path")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|path| path.ends_with(".vcf.gz.tbi")),
        "VCF index probe must retain the indexed governed cohort input"
    );

    let sites_bed = rows
        .iter()
        .find(|row| {
            row.get("missing_input_role").and_then(serde_json::Value::as_str) == Some("sites_bed")
        })
        .expect("sites bed row");
    assert_eq!(
        sites_bed.get("contract_surface").and_then(serde_json::Value::as_str),
        Some("fixture_support")
    );
    assert_eq!(
        sites_bed.get("tool_id").and_then(serde_json::Value::as_str),
        Some("fixture_contract")
    );
    assert!(
        sites_bed
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|reason| reason.contains("no retained VCF adapter consumes it directly")),
        "sites BED coverage must stay explicit about fixture-contract ownership"
    );
}
