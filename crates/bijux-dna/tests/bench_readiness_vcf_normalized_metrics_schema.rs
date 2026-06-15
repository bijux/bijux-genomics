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
fn bench_readiness_vcf_normalized_metrics_schema_reports_governed_stage_extensions() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-vcf-normalized-metrics-schema", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_normalized_metrics_schema.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/schemas/vcf-normalized-metrics.v1.json")
    );
    assert_eq!(
        payload.get("stage_dir").and_then(serde_json::Value::as_str),
        Some("benchmarks/schemas/vcf-normalized-metrics")
    );
    assert_eq!(
        payload.get("schema_id").and_then(serde_json::Value::as_str),
        Some("bijux.schemas.bench.vcf-normalized-metrics.v1")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("extension_count").and_then(serde_json::Value::as_u64), Some(20));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 20);
    assert!(rows.iter().all(|row| {
        row.get("schema_id")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("file_name")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value.ends_with(".v1.json"))
            && row
                .get("extension_id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("required_key_count")
                .and_then(serde_json::Value::as_u64)
                .is_some_and(|value| value > 3)
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca")
            && row.get("schema_version").and_then(serde_json::Value::as_str)
                == Some("bijux.vcf.pca.v1")
            && row.get("file_name").and_then(serde_json::Value::as_str) == Some("pca.v1.json")
            && row.get("extension_id").and_then(serde_json::Value::as_str)
                == Some("vcf_pca_normalized_v1")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.admixture")
            && row.get("schema_version").and_then(serde_json::Value::as_str)
                == Some("bijux.vcf.admixture.v1")
            && row.get("file_name").and_then(serde_json::Value::as_str) == Some("admixture.v1.json")
            && row.get("extension_id").and_then(serde_json::Value::as_str)
                == Some("vcf_admixture_normalized_v1")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.gl_propagation")
            && row.get("schema_version").and_then(serde_json::Value::as_str)
                == Some("bijux.vcf.gl_propagation.v1")
    }));
}
