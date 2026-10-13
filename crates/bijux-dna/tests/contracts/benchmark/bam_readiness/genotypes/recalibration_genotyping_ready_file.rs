#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;

use crate::support;

#[test]
fn bench_readiness_bam_recalibration_genotyping_ready_writes_governed_json_file() {
    let sandbox = support::RepoSandbox::new("bam-recalibration-genotyping-ready-file-")
        .expect("repo sandbox");
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
        .args(["bench", "readiness", "render-bam-recalibration-genotyping-ready"])
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
        "artifacts/benchmark-smoke/readiness/bam/recalibration-genotyping-ready.json"
    );

    let payload = fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read BAM recalibration/genotyping readiness JSON");
    let report: serde_json::Value =
        serde_json::from_str(&payload).expect("parse BAM recalibration/genotyping readiness JSON");

    #[cfg(feature = "bam_downstream")]
    let expected_row_count = 2;
    #[cfg(not(feature = "bam_downstream"))]
    let expected_row_count = 1;

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_recalibration_genotyping_ready.v1")
    );
    assert_eq!(
        report.get("complete_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_row_count)
    );
    assert_eq!(report.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));

    let expected_tool_ids = report
        .get("expected_tool_ids_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("expected tool ids by stage");
    assert!(expected_tool_ids
        .get("bam.recalibration")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|tools| tools.iter().any(|tool| tool.as_str() == Some("gatk"))));

    #[cfg(feature = "bam_downstream")]
    assert!(expected_tool_ids
        .get("bam.genotyping")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|tools| tools.iter().any(|tool| tool.as_str() == Some("bcftools"))));

    let expected_local_proof_kind = report
        .get("expected_local_proof_kind_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("expected local proof kind by stage");
    assert_eq!(
        expected_local_proof_kind.get("bam.recalibration").and_then(serde_json::Value::as_str),
        Some("local_smoke")
    );

    #[cfg(feature = "bam_downstream")]
    assert_eq!(
        expected_local_proof_kind.get("bam.genotyping").and_then(serde_json::Value::as_str),
        Some("local_ready_plan")
    );

    let required_output_ids = report
        .get("required_output_ids_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("required output ids by stage");
    assert!(required_output_ids
        .get("bam.recalibration")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| {
            outputs.iter().any(|output| output.as_str() == Some("recal_bam"))
                && outputs.iter().any(|output| output.as_str() == Some("recal_report"))
                && outputs.iter().any(|output| output.as_str() == Some("stage_metrics"))
        }));

    #[cfg(feature = "bam_downstream")]
    assert!(required_output_ids
        .get("bam.genotyping")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| {
            outputs.iter().any(|output| output.as_str() == Some("genotyping_report"))
                && outputs.iter().any(|output| output.as_str() == Some("summary"))
                && outputs.iter().any(|output| output.as_str() == Some("stage_metrics"))
        }));
}
