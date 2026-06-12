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
fn bench_local_fake_run_all_domains_json_reports_governed_result_slice() {
    let payload = run_cli_json(&["bench", "local", "fake-run-all-domains", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_all_domain_fake_runs.v1")
    );
    assert_eq!(
        payload.get("fake_run_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-fake-runs/all-domains")
    );
    let result_count = support::json_u64(&payload, "result_count").expect("result_count");
    assert!(payload
        .get("created_output_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 120));

    let domain_counts = support::json_object(&payload, "domain_counts");
    assert_eq!(support::object_u64(domain_counts, "fastq"), Some(66));
    assert_eq!(support::object_u64(domain_counts, "bam"), Some(49));
    assert_eq!(support::object_u64_sum(domain_counts), result_count);

    let results = support::json_array(&payload, "results");
    assert_eq!(results.len() as u64, result_count);
    let result_ids = results
        .iter()
        .filter_map(|row| row.get("result_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(result_ids.len() as u64, result_count);

    let taxonomy = results
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
        })
        .expect("taxonomy row");
    assert_eq!(
        taxonomy.get("command_source").and_then(serde_json::Value::as_str),
        Some("fastq_bam_command_adapter")
    );
    assert_eq!(
        taxonomy.get("declared_output_count").and_then(serde_json::Value::as_u64),
        taxonomy.get("created_output_count").and_then(serde_json::Value::as_u64)
    );
    assert!(taxonomy
        .get("expected_metric_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 1));

    let vcf_call = results
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
    assert!(vcf_call.get("outputs").and_then(serde_json::Value::as_array).is_some_and(|outputs| {
        outputs.iter().any(|output| {
            output.get("artifact_id").and_then(serde_json::Value::as_str) == Some("called_vcf")
                && output.get("exists").and_then(serde_json::Value::as_bool) == Some(true)
        }) && outputs.iter().any(|output| {
            output.get("artifact_id").and_then(serde_json::Value::as_str) == Some("called_vcf_tbi")
                && output.get("exists").and_then(serde_json::Value::as_bool) == Some(true)
        })
    }));
    assert!(vcf_call
        .get("stage_result_path")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|path| path.ends_with(
            "/all-domains/vcf/vcf_production_regression/vcf.call/bam_bundle/bcftools/stage-result.json"
        )));
}
