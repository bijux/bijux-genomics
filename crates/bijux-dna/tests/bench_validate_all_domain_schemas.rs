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
fn bench_validate_all_domain_schemas_reports_benchmark_root_pass_state() {
    let payload = run_cli_json(&[
        "bench",
        "validate-schemas",
        "--schema-root",
        "benchmarks/schemas",
        "--domain",
        "fastq,bam,vcf",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_schema_validation.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domain-schema-validation.json")
    );
    assert_eq!(
        payload.get("schema_root").and_then(serde_json::Value::as_str),
        Some("benchmarks/schemas")
    );
    assert_eq!(payload.get("domain_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("passed_domain_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("failed_domain_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok"), Some(&serde_json::Value::Bool(true)));

    let domains =
        payload.get("domains").and_then(serde_json::Value::as_array).expect("domains array");
    assert_eq!(domains.len(), 3);
    assert!(domains.iter().all(|row| {
        row.get("passes_gate") == Some(&serde_json::Value::Bool(true))
            && row.get("shared_schema_matches") == Some(&serde_json::Value::Bool(true))
    }));
    assert!(domains.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("shared_schema_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/schemas/fastq-normalized-metrics.v1.json")
            && row.get("stage_count").and_then(serde_json::Value::as_u64) == Some(27)
    }));
    assert!(domains.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
            && row.get("shared_schema_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/schemas/bam-normalized-metrics.v1.json")
            && row.get("stage_count").and_then(serde_json::Value::as_u64) == Some(24)
    }));
    assert!(domains.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("shared_schema_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/schemas/vcf-normalized-metrics.v1.json")
            && row.get("stage_dir").and_then(serde_json::Value::as_str)
                == Some("benchmarks/schemas/vcf-normalized-metrics")
            && row.get("stage_count").and_then(serde_json::Value::as_u64) == Some(20)
            && row.get("required_stage_count").and_then(serde_json::Value::as_u64) == Some(17)
    }));
}
