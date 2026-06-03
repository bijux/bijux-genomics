#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

#[cfg(feature = "bam_downstream")]
fn run_bam_downstream_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new("cargo")
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["run", "-q", "-p", "bijux-dna", "--features", "bam_downstream", "--"])
        .args(args)
        .output()
        .expect("run cargo cli with bam_downstream")
}

#[cfg(feature = "bam_downstream")]
fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_bam_downstream_cli(args);
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
fn bench_local_validate_slurm_shell_syntax_refuses_invalid_sbatch_syntax() {
    let fixture_root = tempfile::tempdir().expect("fixture root");
    let script_root = fixture_root.path().join("slurm-dry-run");
    std::fs::create_dir_all(&script_root).expect("create script root");
    let bad_script = script_root.join("broken.sbatch");
    std::fs::write(
        &bad_script,
        "#!/usr/bin/env bash\nset -euo pipefail\nif [ -n \"broken\" ]; then\necho still-open\n",
    )
    .expect("write bad script");
    let report_path = fixture_root.path().join("bash-n-report.json");

    let output = run_cli(&[
        "bench",
        "local",
        "validate-slurm-shell-syntax",
        "--root",
        script_root.to_str().expect("script root utf-8"),
        "--output",
        report_path.to_str().expect("report path utf-8"),
        "--json",
    ]);
    assert!(!output.status.success(), "command should fail on invalid sbatch syntax");

    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read report"))
            .expect("parse report json");
    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_slurm_shell_syntax.v1")
    );
    assert_eq!(report.get("script_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(false));
    assert_eq!(report.get("findings_count").and_then(serde_json::Value::as_u64), Some(1));

    let scripts =
        report.get("scripts").and_then(serde_json::Value::as_array).expect("scripts array");
    assert_eq!(scripts.len(), 1);
    assert_eq!(scripts[0].get("ok").and_then(serde_json::Value::as_bool), Some(false));
    assert!(
        scripts[0].get("findings").and_then(serde_json::Value::as_array).is_some_and(|findings| {
            findings.iter().any(|finding| {
                finding.as_str().is_some_and(|text| {
                    text.contains("syntax error") || text.contains("unexpected end of file")
                })
            })
        }),
        "report must capture the bash -n syntax failure"
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_validate_slurm_shell_syntax_accepts_governed_fastq_and_bam_scripts() {
    let fastq_render = run_bam_downstream_cli(&[
        "bench",
        "local",
        "render-slurm-scripts",
        "--domain",
        "fastq",
        "--json",
    ]);
    assert!(
        fastq_render.status.success(),
        "FASTQ render failed: {}\nstdout:\n{}\nstderr:\n{}",
        fastq_render.status,
        String::from_utf8_lossy(&fastq_render.stdout),
        String::from_utf8_lossy(&fastq_render.stderr)
    );

    let bam_render = run_bam_downstream_cli(&[
        "bench",
        "local",
        "render-slurm-scripts",
        "--domain",
        "bam",
        "--json",
    ]);
    assert!(
        bam_render.status.success(),
        "BAM render failed: {}\nstdout:\n{}\nstderr:\n{}",
        bam_render.status,
        String::from_utf8_lossy(&bam_render.stdout),
        String::from_utf8_lossy(&bam_render.stderr)
    );

    let payload = run_cli_json(&["bench", "local", "validate-slurm-shell-syntax", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_slurm_shell_syntax.v1")
    );
    assert_eq!(
        payload.get("root_path").and_then(serde_json::Value::as_str),
        Some("target/slurm-dry-run")
    );
    assert_eq!(payload.get("script_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(payload.get("findings_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let scripts =
        payload.get("scripts").and_then(serde_json::Value::as_array).expect("scripts array");
    assert_eq!(scripts.len(), 51);
    assert!(scripts.iter().all(|entry| {
        entry.get("ok").and_then(serde_json::Value::as_bool) == Some(true)
            && entry.get("script_path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                path.starts_with("target/slurm-dry-run/") && path.ends_with(".sbatch")
            })
    }));
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_validate_slurm_shell_syntax_writes_governed_report_path() {
    let fastq_render = run_bam_downstream_cli(&[
        "bench",
        "local",
        "render-slurm-scripts",
        "--domain",
        "fastq",
        "--json",
    ]);
    assert!(fastq_render.status.success(), "FASTQ render must succeed before syntax validation");

    let bam_render = run_bam_downstream_cli(&[
        "bench",
        "local",
        "render-slurm-scripts",
        "--domain",
        "bam",
        "--json",
    ]);
    assert!(bam_render.status.success(), "BAM render must succeed before syntax validation");

    let payload = run_cli_json(&["bench", "local", "validate-slurm-shell-syntax", "--json"]);
    assert_eq!(
        payload.get("report_path").and_then(serde_json::Value::as_str),
        Some("target/slurm-dry-run/bash-n-report.json")
    );

    let repo_root = support::repo_root().expect("repo root");
    let report_path = repo_root.join("target/slurm-dry-run/bash-n-report.json");
    assert!(report_path.is_file(), "governed bash-n report must exist on disk");

    let written_payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read governed report"))
            .expect("parse governed report");
    assert_eq!(written_payload.get("script_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(written_payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
}
