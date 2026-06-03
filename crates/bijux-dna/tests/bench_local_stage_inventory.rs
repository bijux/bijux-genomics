#![allow(clippy::expect_used)]

use std::path::PathBuf;
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

fn run_cli_json_with_repo_root(args: &[&str]) -> (PathBuf, serde_json::Value) {
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

    (repo_root, serde_json::from_slice(&output.stdout).expect("parse stdout as json"))
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

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_render_stage_commands_json_reports_governed_51_command_slice() {
    let payload = run_cli_json(&["bench", "local", "render-stage-commands", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_stage_commands.v2")
    );
    assert_eq!(
        payload.get("script_output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/rendered-stage-commands.sh")
    );
    assert_eq!(
        payload.get("manifest_output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/rendered-stage-commands.json")
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
            .all(|entry| {
                entry.get("stage_id").and_then(serde_json::Value::as_str).is_some()
                    && entry.get("tool_id").and_then(serde_json::Value::as_str).is_some()
                    && entry.get("threads").and_then(serde_json::Value::as_u64).is_some()
                    && entry.get("memory_mb").and_then(serde_json::Value::as_u64).is_some()
                    && entry.get("command").and_then(serde_json::Value::as_str).is_some()
                    && entry
                        .get("inputs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|inputs| !inputs.is_empty())
                    && entry
                        .get("outputs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|outputs| !outputs.is_empty())
            }),
        "every rendered command row must carry tool, IO, resource, and command fields"
    );
    assert!(
        payload
            .get("commands")
            .and_then(serde_json::Value::as_array)
            .expect("commands array")
            .iter()
            .any(|entry| {
                entry.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
            }),
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
fn bench_local_materialize_stage_bam_validate_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.validate",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.validate"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.validate/validation.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let summary: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.validate validation summary"),
    )
    .expect("parse bam.validate validation summary");

    assert_eq!(
        summary.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.validate.local_smoke.report.v1")
    );
    assert_eq!(summary.get("case_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(summary.get("all_cases_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert!(
        summary.get("cases").and_then(serde_json::Value::as_array).is_some_and(|cases| {
            cases.iter().any(|case| {
                case.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("core-v1-coordinate-pass")
                    && case.get("validation_status").and_then(serde_json::Value::as_str)
                        == Some("pass")
            }) && cases.iter().any(|case| {
                case.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("core-v1-malformed-refusal")
                    && case.get("validation_status").and_then(serde_json::Value::as_str)
                        == Some("refusal")
                    && case.get("refusal_codes").and_then(serde_json::Value::as_array).is_some_and(
                        |codes| codes.contains(&serde_json::json!("malformed_alignment_record")),
                    )
            })
        }),
        "bam.validate local smoke summary must cover governed pass and refusal cases"
    );
}

#[cfg(feature = "bam_downstream")]
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
    let manifest_path = repo_root.join("target/local-ready/rendered-stage-commands.json");
    assert!(script_path.is_file(), "rendered script must exist");
    assert!(manifest_path.is_file(), "rendered JSON manifest must exist");

    let syntax = Command::new("bash").arg("-n").arg(&script_path).output().expect("run bash -n");
    assert!(syntax.status.success(), "bash -n failed: {}", String::from_utf8_lossy(&syntax.stderr));

    let script = std::fs::read_to_string(&script_path).expect("read rendered script");
    assert_eq!(
        script.lines().filter(|line| line.starts_with("cargo run -q -p bijux-dna")).count(),
        51
    );
}
