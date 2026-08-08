#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
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
fn bench_readiness_fastq_local_container_smoke_reports_retained_wrapper_paths() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-fastq-local-container-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_local_container_smoke.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq/fastq-local-container-smoke.tsv")
    );
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let row_count = u64::try_from(rows.len()).expect("row count fits u64");
    let stage_count = u64::try_from(
        rows.iter()
            .filter_map(|row| row.get("stage_id").and_then(serde_json::Value::as_str))
            .collect::<BTreeSet<_>>()
            .len(),
    )
    .expect("stage count fits u64");
    let tool_count = u64::try_from(
        rows.iter()
            .filter_map(|row| row.get("tool_id").and_then(serde_json::Value::as_str))
            .collect::<BTreeSet<_>>()
            .len(),
    )
    .expect("tool count fits u64");
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(row_count));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(stage_count));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(tool_count));
    assert_eq!(
        payload.get("host_stage_smoke_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("container_smoke_row_count").and_then(serde_json::Value::as_u64),
        Some(row_count - 1)
    );
    assert_eq!(
        payload.get("external_artifact_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let runtime_counts = payload
        .get("runtime_counts")
        .and_then(serde_json::Value::as_object)
        .expect("runtime counts");
    let docker_row_count =
        runtime_counts.get("docker-arm64").and_then(serde_json::Value::as_u64).unwrap_or_default();
    let apptainer_row_count =
        runtime_counts.get("apptainer").and_then(serde_json::Value::as_u64).unwrap_or_default();
    assert_eq!(docker_row_count + apptainer_row_count, row_count - 1);
    assert_eq!(runtime_counts.get("external").and_then(serde_json::Value::as_u64), Some(1));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.remove_duplicates")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastuniq")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("external_artifact_admission")
            && row.get("smoke_runtime").and_then(serde_json::Value::as_str) == Some("external")
            && row.get("smoke_support_path").and_then(serde_json::Value::as_str)
                == Some("containers/contracts/catalog.toml")
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str)
            == Some("fastq.detect_duplicates_premerge")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
            && row.get("registered_binary").and_then(serde_json::Value::as_str) == Some("bijux-dna")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("docker_container_smoke")
            && row.get("smoke_runtime").and_then(serde_json::Value::as_str) == Some("docker-arm64")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna env smoke docker-arm64 bijux-dna")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.normalize_primers")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("cutadapt")
            && row.get("registered_binary").and_then(serde_json::Value::as_str) == Some("cutadapt")
            && row.get("smoke_runtime").and_then(serde_json::Value::as_str) == Some("docker-arm64")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna env smoke docker-arm64 cutadapt")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.infer_asvs")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("dada2")
            && row.get("support_status").and_then(serde_json::Value::as_str)
                == Some("governed_execution")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna env smoke docker-arm64 dada2")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.normalize_abundance")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqkit")
            && row.get("support_status").and_then(serde_json::Value::as_str)
                == Some("governed_benchmark_cohort")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna env smoke docker-arm64 seqkit")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str)
            == Some("fastq.profile_read_lengths")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqfu")
            && row.get("registered_binary").and_then(serde_json::Value::as_str) == Some("seqfu")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("support_status").and_then(serde_json::Value::as_str)
                == Some("governed_benchmark_cohort")
            && row.get("smoke_runtime").and_then(serde_json::Value::as_str) == Some("docker-arm64")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna env smoke docker-arm64 seqfu")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.validate_reads")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastq_scan")
            && row.get("registered_binary").and_then(serde_json::Value::as_str)
                == Some("fastq_scan")
            && row.get("support_status").and_then(serde_json::Value::as_str)
                == Some("observer_specialized_benchmark")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna env smoke docker-arm64 fastq_scan")
    }));
}
