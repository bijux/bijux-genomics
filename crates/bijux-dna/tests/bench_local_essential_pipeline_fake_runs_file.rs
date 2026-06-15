#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_fake_run_essential_pipelines_writes_governed_artifact_tree() {
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
        .args(["bench", "local", "fake-run-essential-pipelines"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_root = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_root.trim(), "runs/bench/local-fake-runs/pipelines/essential");

    let fake_run_root = repo_root.join(rendered_root.trim());
    let root_manifest = fake_run_root.join("manifest.json");
    assert!(root_manifest.is_file(), "root manifest must exist");

    let root_manifest_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&root_manifest).expect("read root manifest"))
            .expect("parse root manifest");
    assert_eq!(
        root_manifest_json.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_essential_pipeline_fake_runs.v1")
    );

    let core_align_root = fake_run_root.join("core-germline-fastq-bam-vcf/bam.align");
    assert!(core_align_root.join("command.sh").is_file(), "bam.align command must exist");
    assert!(core_align_root.join("stdout.txt").is_file(), "bam.align stdout must exist");
    assert!(core_align_root.join("stderr.txt").is_file(), "bam.align stderr must exist");
    assert!(core_align_root.join("metrics.json").is_file(), "bam.align metrics must exist");
    assert!(
        core_align_root.join("stage-result.json").is_file(),
        "bam.align stage-result must exist"
    );
    assert!(
        core_align_root.join("declared-outputs/aligned.bam").is_file(),
        "bam.align BAM output must exist"
    );
    assert!(
        core_align_root.join("declared-outputs/align_metrics.json").is_file(),
        "bam.align metrics output must exist"
    );

    let stage_result: serde_json::Value = serde_json::from_slice(
        &std::fs::read(core_align_root.join("stage-result.json")).expect("read stage-result"),
    )
    .expect("parse stage-result");
    assert_eq!(
        stage_result.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(
        stage_result
            .get("runtime")
            .and_then(|value| value.get("mode"))
            .and_then(serde_json::Value::as_str),
        Some("pipeline_fake_run")
    );
    assert_eq!(
        stage_result
            .get("runtime")
            .and_then(|value| value.get("status"))
            .and_then(serde_json::Value::as_str),
        Some("succeeded")
    );

    let impute_root = fake_run_root.join("reference-panel-imputation/vcf.impute");
    assert!(
        impute_root.join("declared-outputs/imputed.vcf").is_file(),
        "vcf.impute VCF output must exist"
    );
    assert!(
        impute_root.join("declared-outputs/imputation_manifest.json").is_file(),
        "vcf.impute manifest output must exist"
    );

    let qc_bundle_root =
        fake_run_root.join("edna-taxonomy-no-vcf/fastq.report_qc/declared-outputs/qc_bundle");
    assert!(qc_bundle_root.is_dir(), "fastq.report_qc bundle output must exist");
    assert!(
        qc_bundle_root.join(".bijux-pipeline-fake-run-placeholder").is_file(),
        "fastq.report_qc bundle sentinel must exist"
    );
}
