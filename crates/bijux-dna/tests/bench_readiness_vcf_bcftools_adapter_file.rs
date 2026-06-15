#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_bcftools_adapter_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-bcftools-adapter"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report_path = repo_root.join("benchmarks/readiness/adapters/bcftools.vcf.json");
    assert!(report_path.is_file(), "VCF bcftools adapter JSON must exist");

    let payload = serde_json::from_slice::<serde_json::Value>(
        &std::fs::read(&report_path).expect("read VCF bcftools adapter JSON"),
    )
    .expect("parse VCF bcftools adapter JSON");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_bcftools_adapter.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(11));
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");

    for stage_id in [
        "vcf.call",
        "vcf.call_diploid",
        "vcf.call_gl",
        "vcf.call_pseudohaploid",
        "vcf.damage_filter",
        "vcf.filter",
        "vcf.gl_propagation",
        "vcf.postprocess",
        "vcf.prepare_reference_panel",
        "vcf.qc",
        "vcf.stats",
    ] {
        assert!(
            rows.iter().any(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
            }),
            "governed adapter JSON must retain stage row: {stage_id}"
        );
    }

    let panel_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.prepare_reference_panel")
        })
        .expect("panel row");
    assert_eq!(
        panel_row.get("parser_output_ids").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("chunks_json".to_string())])
    );

    let stats_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats"))
        .expect("stats row");
    assert_eq!(
        stats_row
            .get("declared_outputs")
            .and_then(serde_json::Value::as_array)
            .map(std::vec::Vec::len),
        Some(2),
        "stats row must retain both raw bcftools output and normalized parser output declarations"
    );

    let qc_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc"))
        .expect("qc row");
    assert_eq!(
        qc_row
            .get("declared_outputs")
            .and_then(serde_json::Value::as_array)
            .map(std::vec::Vec::len),
        Some(7),
        "qc row must retain six raw QC declarations plus the normalized QC report"
    );
}
