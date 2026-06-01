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

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli");

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
fn bench_local_stage_inventory_fastq_json_reports_governed_27_stage_slice() {
    let payload = run_cli_json(&["bench", "local", "list-stages", "--domain", "fastq", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_stage_inventory.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("fastq"));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(27));
    assert_eq!(
        payload.get("stages").and_then(serde_json::Value::as_array).map(std::vec::Vec::len),
        Some(27)
    );
}

#[test]
fn bench_local_stage_inventory_bam_json_reports_governed_24_stage_slice() {
    let payload = run_cli_json(&["bench", "local", "list-stages", "--domain", "bam", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_stage_inventory.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("bam"));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(
        payload.get("stages").and_then(serde_json::Value::as_array).map(std::vec::Vec::len),
        Some(24)
    );
}

#[test]
fn bench_local_render_stage_commands_json_reports_governed_51_command_slice() {
    let payload = run_cli_json(&["bench", "local", "render-stage-commands", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_stage_commands.v1")
    );
    assert_eq!(payload.get("command_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(
        payload.get("commands").and_then(serde_json::Value::as_array).map(std::vec::Vec::len),
        Some(51)
    );
    assert!(
        payload
            .get("commands")
            .and_then(serde_json::Value::as_array)
            .expect("commands array")
            .iter()
            .any(|entry| entry.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.report_qc")),
        "rendered command inventory must include the governed report_qc stage"
    );
}

#[test]
fn bench_local_materialize_stage_report_qc_json_writes_governed_smoke_bundle() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "fastq.report_qc",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.report_qc")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/fastq.report_qc/report.json")
    );
}

#[test]
fn bench_local_render_stage_commands_writes_bash_parseable_51_command_script() {
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
        .args(["bench", "local", "render-stage-commands"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let script_path = repo_root.join("target/local-ready/rendered-stage-commands.sh");
    assert!(script_path.is_file(), "rendered script must exist");

    let syntax = Command::new("bash").arg("-n").arg(&script_path).output().expect("run bash -n");
    assert!(syntax.status.success(), "bash -n failed: {}", String::from_utf8_lossy(&syntax.stderr));

    let script = std::fs::read_to_string(&script_path).expect("read rendered script");
    assert_eq!(
        script.lines().filter(|line| line.starts_with("cargo run -q -p bijux-dna")).count(),
        51
    );
}
