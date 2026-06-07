#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_descent_family_adapter_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-descent-family-adapter"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report_path = repo_root.join("benchmarks/readiness/adapters/descent-family.vcf.json");
    assert!(report_path.is_file(), "descent-family adapter JSON must exist");

    let payload = serde_json::from_slice::<serde_json::Value>(
        &std::fs::read(&report_path).expect("read descent adapter JSON"),
    )
    .expect("parse descent adapter JSON");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_descent_family_adapter.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(payload.get("parser_output_row_count").and_then(serde_json::Value::as_u64), Some(5));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");

    let germline = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("germline")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.ibd")
        })
        .expect("germline row");
    let germline_outputs = germline
        .get("declared_outputs")
        .and_then(serde_json::Value::as_array)
        .expect("germline outputs")
        .iter()
        .filter_map(|artifact| artifact.get("artifact_id"))
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    for artifact_id in ["cohort_bed", "cohort_bim", "cohort_fam", "germline_match", "ibd_segments"]
    {
        assert!(
            germline_outputs.iter().any(|candidate| candidate == &artifact_id),
            "germline row must declare `{artifact_id}`"
        );
    }

    let demography = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("ibdne")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.demography")
        })
        .expect("ibdne row");
    let demography_outputs = demography
        .get("declared_outputs")
        .and_then(serde_json::Value::as_array)
        .expect("demography outputs")
        .iter()
        .filter_map(|artifact| artifact.get("artifact_id"))
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    for artifact_id in ["ne_trajectory_tsv", "demography_report", "logs_txt"] {
        assert!(
            demography_outputs.iter().any(|candidate| candidate == &artifact_id),
            "demography row must declare `{artifact_id}`"
        );
    }

    let demography_input = repo_root.join(
        demography
            .get("input_ibd_segments_path")
            .and_then(serde_json::Value::as_str)
            .expect("demography input path"),
    );
    assert!(
        demography_input.is_file(),
        "demography row must materialize the governed IBD segments input"
    );
}
