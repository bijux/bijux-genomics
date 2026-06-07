#![allow(clippy::expect_used)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn config_write_campaign_profiles_emits_lunarc_fastq_bam_vcf_local_ready_profile() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let out_dir = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args([
            "config",
            "write-campaign-profiles",
            "--out-dir",
            out_dir.path().to_str().expect("utf8 path"),
        ])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("written="));
    assert!(stdout.contains("lunarc-fastq-bam-vcf-local-ready.toml"));

    let rendered_path = out_dir.path().join("lunarc-fastq-bam-vcf-local-ready.toml");
    assert!(rendered_path.is_file(), "generated profile must exist");

    let rendered = fs::read_to_string(&rendered_path).expect("read generated profile");
    assert!(rendered.contains("id = \"adna-equus-caballus-fastq-bam-vcf-local-ready\""));
    assert!(rendered.contains("stage = \"vcf.call\""));
    assert!(rendered.contains("stage = \"vcf.stats\""));
    assert!(rendered.contains("stage = \"vcf.qc\""));
    assert!(!rendered.contains("stage = \"fastq.index_reference\""));
    assert!(!rendered.contains("stage = \"vcf.prepare_reference_panel\""));

    let temp = tempfile::tempdir().expect("tempdir");
    let env_file = temp.path().join("missing.env");
    let policy_file = temp.path().join("missing.policy");
    let dry_run = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args([
            "config",
            "campaign-dry-run",
            "--config",
            rendered_path.to_str().expect("utf8 path"),
            "--env-file",
            env_file.to_str().expect("utf8 path"),
            "--user-policies",
            policy_file.to_str().expect("utf8 path"),
            "--json",
        ])
        .output()
        .expect("run dry run");

    assert!(
        dry_run.status.success(),
        "dry run failed: {}\nstdout:\n{}\nstderr:\n{}",
        dry_run.status,
        String::from_utf8_lossy(&dry_run.stdout),
        String::from_utf8_lossy(&dry_run.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&dry_run.stdout).expect("parse dry run output");
    assert_eq!(
        payload.get("campaign_id").and_then(serde_json::Value::as_str),
        Some("adna-equus-caballus-fastq-bam-vcf-local-ready")
    );
    assert_eq!(
        payload.get("planned_jobs").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(9)
    );
}
