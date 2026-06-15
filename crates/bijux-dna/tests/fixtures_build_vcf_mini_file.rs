#![allow(clippy::expect_used)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> (std::path::PathBuf, std::process::Output) {
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
        .args(args)
        .output()
        .expect("run cli");

    (repo_root, output)
}

#[test]
fn fixtures_build_vcf_mini_writes_manifest_report_and_fixture_assets() {
    let output_root = "artifacts/fixtures/vcf-mini-regeneration-check";
    let (repo_root, output) =
        run_cli(&["fixtures", "build", "--corpus", "vcf-mini", "--out", output_root]);

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "artifacts/fixtures/vcf-mini-regeneration-check/manifest.json");

    let manifest_report_path = repo_root.join(output_root).join("manifest.json");
    let manifest_report_raw =
        fs::read_to_string(&manifest_report_path).expect("read manifest.json");
    let manifest_report: serde_json::Value =
        serde_json::from_str(&manifest_report_raw).expect("parse manifest.json");

    assert_eq!(
        manifest_report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.vcf_fixture_build.v1")
    );
    assert_eq!(
        manifest_report.get("governed_counts_match").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    assert!(repo_root.join(output_root).join("manifest.toml").exists());
    assert!(repo_root.join(output_root).join("CHECKSUMS.sha256").exists());
    assert!(repo_root.join(output_root).join("reference/vcf_mini_reference.fasta").exists());
    assert!(repo_root.join(output_root).join("variants/vcf_mini_multisample.vcf").exists());
    assert!(repo_root.join(output_root).join("expected/variant_counts.json").exists());
}
