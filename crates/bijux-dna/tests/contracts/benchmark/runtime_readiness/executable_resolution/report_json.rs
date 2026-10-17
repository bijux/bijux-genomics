#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(support::cli_binary_path())
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
        Some("artifacts/benchmark-smoke/readiness/tools/executable-resolution.tsv")
    );
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    let row_count = rows.len() as u64;
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(row_count));

    let resolution_counts = payload
        .get("resolution_counts")
        .and_then(serde_json::Value::as_object)
        .expect("resolution counts");
    assert_eq!(resolution_counts.get("host_binary").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        resolution_counts.get("apptainer_image").and_then(serde_json::Value::as_u64),
        Some(12)
    );
    assert_eq!(
        resolution_counts.get("external_artifact").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert!(resolution_counts.get("unavailable_with_reason").is_none());
    assert_eq!(payload.get("unavailable_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        resolution_counts.values().filter_map(serde_json::Value::as_u64).sum::<u64>(),
        row_count
    );

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
                    "containers/apptainer/beagle.def@sha256:220b8f1687f32f6f04cb4e85b0d6ab4ecd2e98f6f5147064c4c2420ddfdd5b3f"
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("angsd")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("external_artifact")
            && row.get("resolution_target").and_then(serde_json::Value::as_str)
                == Some("external-artifact://angsd")
            && row.get("unavailable_reason").is_none_or(serde_json::Value::is_null)
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastuniq")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("external_artifact")
            && row.get("resolution_target").and_then(serde_json::Value::as_str)
                == Some("external-artifact://fastuniq")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("impute5")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("external_artifact")
            && row.get("resolution_target").and_then(serde_json::Value::as_str)
                == Some("external-artifact://impute5")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("ngsbriggs")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("external_artifact")
            && row.get("resolution_target").and_then(serde_json::Value::as_str)
                == Some("external-artifact://ngsbriggs")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit5")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("apptainer_image")
            && row.get("unavailable_reason").is_none_or(serde_json::Value::is_null)
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("apptainer_image")
            && row.get("unavailable_reason").is_none_or(serde_json::Value::is_null)
    }));
    for tool_id in ["eagle", "glimpse", "ibdhap", "minimac4", "shapeit"] {
        assert!(rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                    == Some("apptainer_image")
                && row.get("unavailable_reason").is_none_or(serde_json::Value::is_null)
        }));
    }
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
            && row
                .get("active_stage_ids")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|value| value.as_str() == Some("vcf.roh")))
    }));
}
