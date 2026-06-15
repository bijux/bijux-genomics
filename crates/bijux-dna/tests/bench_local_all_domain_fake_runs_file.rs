#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_fake_run_all_domains_writes_governed_artifact_tree() {
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
        .args(["bench", "local", "fake-run-all-domains"])
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
    assert_eq!(rendered_root.trim(), "runs/bench/local-fake-runs/all-domains");

    let fake_run_root = repo_root.join(rendered_root.trim());
    let root_manifest = fake_run_root.join("manifest.json");
    assert!(root_manifest.is_file(), "root manifest must exist");

    let taxonomy_root = fake_run_root
        .join(
            "fastq/corpus-02-edna-mini/fastq.screen_taxonomy/database_artifact_id+taxonomy_database_root/kraken2",
        );
    assert!(taxonomy_root.join("command.sh").is_file(), "taxonomy command must exist");
    assert!(taxonomy_root.join("stdout.txt").is_file(), "taxonomy stdout must exist");
    assert!(taxonomy_root.join("stderr.txt").is_file(), "taxonomy stderr must exist");
    assert!(taxonomy_root.join("metrics.json").is_file(), "taxonomy metrics must exist");
    assert!(taxonomy_root.join("stage-result.json").is_file(), "taxonomy stage-result must exist");
    assert!(
        taxonomy_root.join("declared-outputs/classification_report.json").is_file(),
        "taxonomy classification report must exist"
    );
    assert!(
        taxonomy_root.join("declared-outputs/screen_report.tsv").is_file(),
        "taxonomy screen report must exist"
    );

    let vcf_call_root =
        fake_run_root.join("vcf/vcf_production_regression/vcf.call/bam_bundle/bcftools");
    assert!(vcf_call_root.join("command.sh").is_file(), "vcf.call command must exist");
    assert!(
        vcf_call_root.join("declared-outputs/called.vcf").is_file(),
        "vcf.call VCF output must exist"
    );
    assert!(
        vcf_call_root.join("declared-outputs/called.vcf.tbi").is_file(),
        "vcf.call index output must exist"
    );

    let stage_result: serde_json::Value = serde_json::from_slice(
        &std::fs::read(vcf_call_root.join("stage-result.json")).expect("read stage-result"),
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
        Some("benchmark_fake_run")
    );
    assert_eq!(
        stage_result
            .get("runtime")
            .and_then(|value| value.get("status"))
            .and_then(serde_json::Value::as_str),
        Some("succeeded")
    );
}
