#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;

use crate::support;

#[test]
fn bench_readiness_bam_kinship_complete_writes_governed_json_file() {
    let sandbox =
        support::RepoSandbox::new_bam_genotype("bam-kinship-complete-file-").expect("repo sandbox");
    let repo_root = sandbox.path();
    let runtime_root = sandbox.path().join("artifacts/test-runtime");
    std::fs::create_dir_all(&runtime_root).expect("private runtime root");
    let home = tempfile::Builder::new()
        .prefix("bam-genotype-runtime-")
        .tempdir_in(&runtime_root)
        .expect("private runtime directory");

    let output = sandbox
        .bijux_dna_command()
        .envs([
            ("HOME", home.path()),
            ("TMPDIR", home.path()),
            ("TMP", home.path()),
            ("TEMP", home.path()),
        ])
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-bam-kinship-complete"])
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
        "artifacts/benchmark-smoke/readiness/bam/stages/bam.kinship.complete.json"
    );

    let payload = fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read BAM kinship completion JSON");
    let report: serde_json::Value =
        serde_json::from_str(&payload).expect("parse BAM kinship completion JSON");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_kinship_complete.v1")
    );
    let rows = report.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(
        report.get("complete_row_count").and_then(serde_json::Value::as_u64),
        Some(rows.len() as u64)
    );
    assert_eq!(report.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(report.get("toolset_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        report.get("required_output_ids").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::json!("kinship_report"),
            serde_json::json!("summary"),
            serde_json::json!("stage_metrics")
        ])
    );
    assert_eq!(
        report.get("expected_tool_ids").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::json!("king")])
    );
    assert_eq!(report.get("local_smoke_case_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        report.get("bam_pipeline_id").and_then(serde_json::Value::as_str),
        Some("bam-kinship")
    );
    assert_eq!(
        report.get("vcf_pipeline_id").and_then(serde_json::Value::as_str),
        Some("bam-genotyping-to-vcf-downstream")
    );
}
