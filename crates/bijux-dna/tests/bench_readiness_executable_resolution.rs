#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_readiness_executable_resolution_reports_governed_runtime_locations() {
    let payload = run_cli_json(&["bench", "readiness", "render-executable-resolution", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.executable_resolution.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/tools/executable-resolution.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(71));

    let resolution_counts = payload
        .get("resolution_counts")
        .and_then(serde_json::Value::as_object)
        .expect("resolution counts");
    assert_eq!(resolution_counts.get("host_binary").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(resolution_counts.get("docker_image").and_then(serde_json::Value::as_u64), Some(67));
    assert_eq!(
        resolution_counts.get("apptainer_image").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        resolution_counts.get("unavailable_with_reason").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(payload.get("unavailable_count").and_then(serde_json::Value::as_u64), Some(1));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 71);

    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str) == Some("host_binary")
            && row.get("resolution_target").and_then(serde_json::Value::as_str) == Some("bijux-dna")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("docker_image")
            && row.get("resolution_target").and_then(serde_json::Value::as_str)
                == Some("bijuxdna/samtools:1.21")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("apptainer_image")
            && row.get("resolution_target").and_then(serde_json::Value::as_str)
                == Some(
                    "containers/apptainer/lunarc/beagle.def@sha256:220b8f1687f32f6f04cb4e85b0d6ab4ecd2e98f6f5147064c4c2420ddfdd5b3f"
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit5")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("unavailable_with_reason")
            && row
                .get("unavailable_reason")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|reason| reason.contains("external container source"))
    }));
}
