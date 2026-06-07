#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let temp = tempfile::tempdir().expect("tempdir");
    let env_file = temp.path().join("missing.env");
    let policy_file = temp.path().join("missing.policy");

    let mut command = Command::new(env!("CARGO_BIN_EXE_bijux-dna"));
    command
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .arg("config")
        .arg("campaign-dry-run")
        .arg("--config")
        .arg("configs/hpc/campaign/lunarc-fastq-bam-vcf-local-ready.toml")
        .arg("--env-file")
        .arg(&env_file)
        .arg("--user-policies")
        .arg(&policy_file)
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args);

    let output = command.output().expect("run cli");
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn config_campaign_dry_run_lunarc_fastq_bam_vcf_local_ready_reports_prepared_cross_domain_jobs() {
    let payload = run_cli_json(&["--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.hpc.campaign.v1")
    );
    assert_eq!(
        payload.get("campaign_id").and_then(serde_json::Value::as_str),
        Some("adna-equus-caballus-fastq-bam-vcf-local-ready")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("cross"));
    assert_eq!(
        payload
            .get("layout")
            .and_then(|layout| layout.get("corpora_root"))
            .and_then(serde_json::Value::as_str),
        Some("/mnt/shared/bijux/corpora")
    );
    assert_eq!(
        payload
            .get("layout")
            .and_then(|layout| layout.get("databases_root"))
            .and_then(serde_json::Value::as_str),
        Some("/mnt/shared/bijux/databases")
    );
    assert_eq!(
        payload
            .get("layout")
            .and_then(|layout| layout.get("images_root"))
            .and_then(serde_json::Value::as_str),
        Some("/mnt/shared/bijux/images")
    );
    assert_eq!(
        payload
            .get("resolved_slurm")
            .and_then(|slurm| slurm.get("site_profile"))
            .and_then(serde_json::Value::as_str),
        Some("lunarc")
    );
    assert_eq!(
        payload
            .get("resolved_slurm")
            .and_then(|slurm| slurm.get("partition"))
            .and_then(serde_json::Value::as_str),
        Some("main")
    );
    assert_eq!(
        payload
            .get("resolved_slurm")
            .and_then(|slurm| slurm.get("qos"))
            .and_then(serde_json::Value::as_str),
        Some("normal")
    );

    let jobs =
        payload.get("planned_jobs").and_then(serde_json::Value::as_array).expect("planned jobs");
    assert_eq!(jobs.len(), 9);

    let stages: Vec<_> = jobs
        .iter()
        .map(|job| job.get("stage").and_then(serde_json::Value::as_str).expect("stage"))
        .collect();
    assert_eq!(
        stages,
        vec![
            "fastq.validate_reads",
            "fastq.trim_reads",
            "bam.align",
            "bam.qc_pre",
            "bam.recalibration",
            "vcf.call",
            "vcf.filter",
            "vcf.stats",
            "vcf.qc",
        ]
    );
    assert!(!stages.iter().any(|stage| matches!(
        *stage,
        "fastq.index_reference" | "vcf.prepare_reference_panel" | "bam.index_reference"
    )));

    let vcf_call = jobs
        .iter()
        .find(|job| job.get("stage").and_then(serde_json::Value::as_str) == Some("vcf.call"))
        .expect("vcf.call job");
    assert_eq!(
        vcf_call.get("resource_template").and_then(serde_json::Value::as_str),
        Some("vcf_call")
    );

    let vcf_qc = jobs
        .iter()
        .find(|job| job.get("stage").and_then(serde_json::Value::as_str) == Some("vcf.qc"))
        .expect("vcf.qc job");
    assert_eq!(vcf_qc.get("resource_template").and_then(serde_json::Value::as_str), Some("vcf_qc"));
}
