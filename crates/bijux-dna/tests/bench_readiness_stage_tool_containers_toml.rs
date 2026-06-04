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

#[test]
fn bench_readiness_stage_tool_containers_writes_governed_toml_file() {
    let output = run_cli(&["bench", "readiness", "render-stage-tool-containers"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let repo_root = support::repo_root().expect("repo root");
    let config_path = repo_root.join("configs/bench/local/stage-tool-containers.toml");
    let raw = std::fs::read_to_string(&config_path).expect("read config");
    let parsed: toml::Value = toml::from_str(&raw).expect("parse config");

    assert_eq!(
        parsed.get("schema_version").and_then(toml::Value::as_str),
        Some("bijux.bench.local_stage_tool_containers.v1")
    );
    assert_eq!(
        parsed.get("classification_scope").and_then(toml::Value::as_str),
        Some("benchmark_ready_runtime_declarations")
    );
    let rows = parsed.get("rows").and_then(toml::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 55);
    assert!(rows.iter().all(|row| {
        row.get("container_id").is_some()
            || row.get("command_entrypoint").is_some()
            || row.get("host_binary_mode").is_some()
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.normalize_primers")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("cutadapt")
            && row
                .get("container_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| value.starts_with("bijuxdna/cutadapt@sha256:"))
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("cutadapt")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.detect_adapters")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("fastqc")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("java")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("fastqc")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some(
                    "bijuxdna/fastqc@sha256:e0b83c56262486cab51020e2bb809b391ad9b38ba7a898588ab15b73586ee789"
                )
    }));
}
