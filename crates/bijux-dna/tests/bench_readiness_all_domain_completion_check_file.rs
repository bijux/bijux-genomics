#![allow(clippy::expect_used)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_completion_check_writes_governed_fixture_tree() {
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
        .args(["bench", "readiness", "render-all-domain-completion-check"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "target/bench-readiness/completion-check-all-domains.json"
    );

    let report_path = repo_root.join("target/bench-readiness/completion-check-all-domains.json");
    assert!(report_path.is_file(), "expected completion-check report");

    let fixture_root =
        repo_root.join("target/bench-readiness/completion-check-all-domains-fixture");
    assert!(fixture_root.is_dir(), "expected completion-check fixture root");

    let missing_manifest = fixture_root
        .join("vcf")
        .join("vcf_production_regression")
        .join("vcf.call")
        .join("bam_bundle")
        .join("bcftools")
        .join("stage-result.json");
    assert!(!missing_manifest.exists(), "seeded VCF manifest must stay absent");

    let empty_command = fixture_root
        .join("bam")
        .join("corpus-01-bam-mini")
        .join("bam.coverage")
        .join("sample-set")
        .join("samtools")
        .join("command.sh");
    assert!(empty_command.is_file(), "expected seeded command script");
    assert_eq!(
        fs::metadata(&empty_command).expect("stat empty command").len(),
        0,
        "seeded command script must stay empty"
    );

    let missing_normalized_metric = fixture_root
        .join("fastq")
        .join("corpus-02-edna-mini")
        .join("fastq.screen_taxonomy")
        .join("sample-set")
        .join("kraken2")
        .join("declared-outputs")
        .join("classification_report.json");
    assert!(!missing_normalized_metric.exists(), "seeded normalized metrics file must stay absent");

    let execution_manifest = fixture_root
        .join("bam")
        .join("corpus-01-bam-mini")
        .join("bam.qc_pre")
        .join("sample-set")
        .join("multiqc")
        .join("stage-result.json");
    let execution_manifest_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&execution_manifest).expect("read execution manifest"))
            .expect("parse execution manifest");
    assert_eq!(
        execution_manifest_json
            .get("runtime")
            .and_then(|value| value.get("exit_code"))
            .and_then(serde_json::Value::as_i64),
        Some(23)
    );
    assert_eq!(
        execution_manifest_json
            .get("runtime")
            .and_then(|value| value.get("status"))
            .and_then(serde_json::Value::as_str),
        Some("failed")
    );

    let missing_declared_output = fixture_root
        .join("fastq")
        .join("corpus-01-mini")
        .join("fastq.profile_reads")
        .join("sample-set")
        .join("seqkit_stats")
        .join("declared-outputs")
        .join("qc.tsv");
    assert!(!missing_declared_output.exists(), "seeded declared output file must stay absent");
}
