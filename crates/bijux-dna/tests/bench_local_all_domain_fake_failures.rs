#![allow(clippy::expect_used)]

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
fn bench_local_fake_run_all_domain_failures_json_reports_governed_result_slice() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "fake-run-all-domain-failures",
        "--exit-code",
        "13",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_all_domain_fake_failures.v1")
    );
    assert_eq!(
        payload.get("failure_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-fake-runs/all-domains-failures")
    );
    assert_eq!(payload.get("result_count").and_then(serde_json::Value::as_u64), Some(121));
    assert_eq!(payload.get("exit_code").and_then(serde_json::Value::as_i64), Some(13));
    assert!(payload
        .get("failed_output_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 120));

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(9));

    let failures =
        payload.get("failures").and_then(serde_json::Value::as_array).expect("failures array");
    assert_eq!(failures.len(), 121);
    let result_ids = failures
        .iter()
        .filter_map(|row| row.get("result_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(result_ids.len(), 121);

    let taxonomy = failures
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
        })
        .expect("taxonomy row");
    assert_eq!(taxonomy.get("exit_code").and_then(serde_json::Value::as_i64), Some(13));
    assert_eq!(
        taxonomy.get("command_source").and_then(serde_json::Value::as_str),
        Some("fastq_bam_command_adapter")
    );
    assert!(taxonomy
        .get("failed_output_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 2));
    assert!(taxonomy.get("failed_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| outputs.iter().any(|output| {
            output.get("artifact_id").and_then(serde_json::Value::as_str)
                == Some("classification_report_json")
        })
    ));

    let vcf_call = failures
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
        })
        .expect("VCF call row");
    assert_eq!(
        vcf_call.get("command_source").and_then(serde_json::Value::as_str),
        Some("vcf_bcftools_adapter")
    );
    assert!(vcf_call.get("failed_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| outputs.iter().any(|output| {
            output.get("artifact_id").and_then(serde_json::Value::as_str) == Some("called_vcf")
        }) && outputs.iter().any(|output| {
            output.get("artifact_id").and_then(serde_json::Value::as_str) == Some("called_vcf_tbi")
        })
    ));
}
