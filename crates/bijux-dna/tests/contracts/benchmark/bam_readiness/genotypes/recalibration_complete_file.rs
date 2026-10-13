#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;

use crate::support;

#[test]
fn bench_readiness_bam_recalibration_complete_writes_governed_json_file() {
    let sandbox =
        support::RepoSandbox::new("bam-recalibration-complete-file-").expect("repo sandbox");
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
        .args(["bench", "readiness", "render-bam-recalibration-complete"])
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
        "artifacts/benchmark-smoke/readiness/bam/stages/bam.recalibration.complete.json"
    );

    let payload = fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read BAM recalibration completion JSON");
    let report: serde_json::Value =
        serde_json::from_str(&payload).expect("parse BAM recalibration completion JSON");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_recalibration_complete.v1")
    );
    assert_eq!(report.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(report.get("toolset_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        report.get("local_smoke_sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_recalibration_low_coverage")
    );

    assert!(report.get("required_output_ids").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| {
            outputs.iter().any(|output| output.as_str() == Some("recal_bam"))
                && outputs.iter().any(|output| output.as_str() == Some("recal_bai"))
                && outputs.iter().any(|output| output.as_str() == Some("recal_report"))
                && outputs.iter().any(|output| output.as_str() == Some("summary"))
                && outputs.iter().any(|output| output.as_str() == Some("stage_metrics"))
        }
    ));
}
