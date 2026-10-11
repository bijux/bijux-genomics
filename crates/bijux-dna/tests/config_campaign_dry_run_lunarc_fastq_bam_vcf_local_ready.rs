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
        .arg("benchmarks/configs/hpc/campaign/lunarc-fastq-bam-vcf-local-ready.toml")
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

fn assert_campaign_identity_and_site(payload: &serde_json::Value) {
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.hpc.campaign.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/configs/hpc/campaign/lunarc-fastq-bam-vcf-local-ready.toml")
    );
    assert_eq!(
        payload.get("campaign_id").and_then(serde_json::Value::as_str),
        Some("adna-equus-caballus-fastq-bam-vcf-local-ready")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("cross"));
    for (key, expected) in [
        ("corpora_root", "/mnt/shared/bijux/corpora"),
        ("databases_root", "/mnt/shared/bijux/databases"),
        ("images_root", "/mnt/shared/bijux/images"),
    ] {
        assert_eq!(
            payload
                .get("layout")
                .and_then(|layout| layout.get(key))
                .and_then(serde_json::Value::as_str),
            Some(expected)
        );
    }
    for (key, expected) in [("site_profile", "lunarc"), ("partition", "main"), ("qos", "normal")] {
        assert_eq!(
            payload
                .get("resolved_slurm")
                .and_then(|slurm| slurm.get(key))
                .and_then(serde_json::Value::as_str),
            Some(expected)
        );
    }
}

#[test]
fn config_campaign_dry_run_lunarc_fastq_bam_vcf_local_ready_reports_prepared_cross_domain_jobs() {
    let payload = run_cli_json(&["--json"]);
    assert_campaign_identity_and_site(&payload);

    let jobs =
        payload.get("planned_jobs").and_then(serde_json::Value::as_array).expect("planned jobs");
    assert_eq!(jobs.len(), 16);

    let stages: Vec<_> = jobs
        .iter()
        .map(|job| job.get("stage").and_then(serde_json::Value::as_str).expect("stage"))
        .collect();
    assert_eq!(
        stages,
        vec![
            "fastq.validate_reads",
            "fastq.profile_read_lengths",
            "fastq.detect_adapters",
            "fastq.trim_reads",
            "fastq.merge_pairs",
            "fastq.filter_reads",
            "bam.align",
            "bam.markdup",
            "bam.qc_pre",
            "bam.damage",
            "bam.coverage",
            "bam.duplication_metrics",
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
    assert_eq!(
        jobs.iter()
            .find(|job| job.get("stage").and_then(serde_json::Value::as_str) == Some("bam.markdup"))
            .and_then(|job| job.get("resource_template"))
            .and_then(serde_json::Value::as_str),
        Some("bam_markdup")
    );

    let vcf_qc = jobs
        .iter()
        .find(|job| job.get("stage").and_then(serde_json::Value::as_str) == Some("vcf.qc"))
        .expect("vcf.qc job");
    assert_eq!(vcf_qc.get("resource_template").and_then(serde_json::Value::as_str), Some("vcf_qc"));
}
