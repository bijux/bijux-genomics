#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_fake_run_all_domain_failures_writes_governed_failure_tree() {
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
        .args(["bench", "local", "fake-run-all-domain-failures", "--exit-code", "9"])
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
    assert_eq!(rendered_root.trim(), "target/local-fake-runs/all-domains-failures");

    let failure_root = repo_root.join(rendered_root.trim());
    let root_manifest = failure_root.join("manifest.json");
    assert!(root_manifest.is_file(), "root manifest must exist");

    let taxonomy_root = failure_root.join(
        "fastq/corpus-02-edna-mini/fastq.screen_taxonomy/database_artifact_id+taxonomy_database_root/kraken2",
    );
    assert!(taxonomy_root.join("command.sh").is_file(), "taxonomy command must exist");
    assert!(taxonomy_root.join("stderr.txt").is_file(), "taxonomy stderr must exist");
    assert!(taxonomy_root.join("failure.json").is_file(), "taxonomy failure record must exist");
    assert!(
        !taxonomy_root.join("declared-outputs/classification_report.json").exists(),
        "taxonomy fake failure must not materialize success outputs"
    );

    let vcf_call_root =
        failure_root.join("vcf/vcf_production_regression/vcf.call/bam_bundle/bcftools");
    assert!(vcf_call_root.join("command.sh").is_file(), "vcf.call command must exist");
    assert!(vcf_call_root.join("stderr.txt").is_file(), "vcf.call stderr must exist");
    assert!(vcf_call_root.join("failure.json").is_file(), "vcf.call failure record must exist");
    assert!(
        !vcf_call_root.join("declared-outputs/called.vcf").exists(),
        "vcf.call fake failure must not materialize success outputs"
    );

    let failure_record: serde_json::Value = serde_json::from_slice(
        &std::fs::read(vcf_call_root.join("failure.json")).expect("read failure record"),
    )
    .expect("parse failure record");
    assert_eq!(
        failure_record.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_all_domain_fake_failure_record.v1")
    );
    assert_eq!(failure_record.get("exit_code").and_then(serde_json::Value::as_i64), Some(9));
    assert!(failure_record
        .get("failed_outputs")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| !outputs.is_empty()));
}
