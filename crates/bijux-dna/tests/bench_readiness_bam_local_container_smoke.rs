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
fn bench_readiness_bam_local_container_smoke_reports_retained_wrapper_paths() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-bam-local-container-smoke", "--json"]);

    let expected_host_count = if cfg!(feature = "bam_downstream") { 20 } else { 18 };
    let expected_container_count = 49 - expected_host_count;

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_local_container_smoke.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/bam-local-container-smoke.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(
        payload.get("host_stage_smoke_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_host_count)
    );
    assert_eq!(
        payload.get("container_smoke_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_container_count)
    );

    let runtime_counts = payload
        .get("runtime_counts")
        .and_then(serde_json::Value::as_object)
        .expect("runtime counts");
    assert_eq!(
        runtime_counts.get("host").and_then(serde_json::Value::as_u64),
        Some(expected_host_count)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 49);

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.validate")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
            && row.get("registered_binary").and_then(serde_json::Value::as_str) == Some("samtools")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("host_stage_smoke")
            && row.get("smoke_runtime").and_then(serde_json::Value::as_str) == Some("host")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna bench local run-bam-stage-smoke --stage-id bam.validate")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.coverage")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("host_stage_smoke")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna bench local run-bam-stage-smoke --stage-id bam.coverage")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.coverage")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("mosdepth")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("docker_container_smoke")
            && row.get("smoke_runtime").and_then(serde_json::Value::as_str) == Some("docker-arm64")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna env smoke docker-arm64 mosdepth")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bwa")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("docker_container_smoke")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna env smoke docker-arm64 bwa")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("verifybamid2")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("docker_container_smoke")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna env smoke docker-arm64 verifybamid2")
    }));
    #[cfg(feature = "bam_downstream")]
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("king")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("host_stage_smoke")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna bench local run-bam-stage-smoke --stage-id bam.kinship")
    }));
}
