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

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
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
fn bench_readiness_stage_tool_containers_reports_governed_runtime_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-stage-tool-containers", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.stage_tool_containers.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/stage-tool-containers.toml")
    );
    assert_eq!(
        payload.get("classification_scope").and_then(serde_json::Value::as_str),
        Some("benchmark_ready_runtime_declarations")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(59));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(59)
    );
    assert_eq!(payload.get("external_row_count").and_then(serde_json::Value::as_u64), Some(59));
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(51)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(8)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert!(rows.iter().all(|row| {
        row.get("container_id").is_some()
            || row.get("command_entrypoint").is_some()
            || row.get("host_binary_mode").is_some()
    }));
    let cutadapt = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.normalize_primers")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("cutadapt")
        })
        .expect("cutadapt normalize_primers row");
    assert_eq!(cutadapt.get("execution_mode").and_then(serde_json::Value::as_str), Some("python"));
    assert_eq!(
        cutadapt.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("cutadapt")
    );
    assert!(
        cutadapt
            .get("container_id")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.starts_with("bijuxdna/cutadapt@sha256:")),
        "cutadapt row must preserve the governed container declaration"
    );
    let fastqc = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.detect_adapters")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastqc")
        })
        .expect("detect-adapters fastqc row");
    assert_eq!(fastqc.get("execution_mode").and_then(serde_json::Value::as_str), Some("java"));
    assert_eq!(
        fastqc.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("fastqc")
    );
    assert_eq!(
        fastqc.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/fastqc@sha256:e0b83c56262486cab51020e2bb809b391ad9b38ba7a898588ab15b73586ee789")
    );
    let fastp = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.filter_reads")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastp")
        })
        .expect("filter-reads fastp row");
    assert_eq!(
        fastp.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(fastp.get("command_entrypoint").and_then(serde_json::Value::as_str), Some("fastp"));
    assert_eq!(
        fastp.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/fastp@sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10")
    );
}
