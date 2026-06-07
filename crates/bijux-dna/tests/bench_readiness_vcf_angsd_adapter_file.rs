#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_angsd_adapter_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-angsd-adapter"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report_path = repo_root.join("target/bench-readiness/adapters/angsd.vcf.json");
    assert!(report_path.is_file(), "VCF angsd adapter JSON must exist");

    let payload = serde_json::from_slice::<serde_json::Value>(
        &std::fs::read(&report_path).expect("read VCF angsd adapter JSON"),
    )
    .expect("parse VCF angsd adapter JSON");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_angsd_adapter.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(4));
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");

    for stage_id in
        ["vcf.call_gl", "vcf.call_pseudohaploid", "vcf.damage_filter", "vcf.gl_propagation"]
    {
        assert!(
            rows.iter().any(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
            }),
            "governed adapter JSON must retain stage row: {stage_id}"
        );
    }

    let call_gl_bam_list =
        repo_root.join("target/bench-readiness/adapters/angsd/vcf.call_gl/angsd-inputs.bam.list");
    assert!(
        call_gl_bam_list.is_file(),
        "call_gl row must materialize the governed bam.list helper"
    );
    assert_eq!(
        std::fs::read_to_string(&call_gl_bam_list).expect("read call_gl bam.list"),
        "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam\n"
    );

    let damage_filter = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.damage_filter")
        })
        .expect("damage_filter row");
    assert_eq!(
        damage_filter.get("raw_output_ids").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("damage_report_txt".to_string()),
            serde_json::Value::String("angsd_arg".to_string()),
        ])
    );

    let gl_propagation = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.gl_propagation")
        })
        .expect("gl_propagation row");
    assert_eq!(
        gl_propagation
            .get("declared_outputs")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.len()),
        Some(3),
        "gl_propagation row must retain raw VCF, angsd arg, and normalized parser output declarations"
    );
}
